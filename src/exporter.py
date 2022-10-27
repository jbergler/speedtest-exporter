import subprocess
import json
import os
import logging
import datetime
from prometheus_client import make_wsgi_app, Gauge, Info
from flask import Flask
from waitress import serve
from shutil import which

app = Flask("Speedtest-Exporter")  # Create flask app

# Setup logging values
format_string = 'level=%(levelname)s datetime=%(asctime)s %(message)s'
logging.basicConfig(encoding='utf-8',
                    level=logging.DEBUG,
                    format=format_string)

# Disable Waitress Logs
log = logging.getLogger('waitress')
log.disabled = True

# Labels for metrics
labels = ["server_id"]

# Create Metrics
jitter = Gauge(
    "speedtest_jitter_latency_milliseconds", "Speedtest current Jitter in ms", labels
)
ping = Gauge(
    "speedtest_ping_latency_milliseconds", "Speedtest current Ping in ms", labels
)
download_speed = Gauge(
    "speedtest_download_bits_per_second",
    "Speedtest current Download Speed in bit/s",
    labels,
)
upload_speed = Gauge(
    "speedtest_upload_bits_per_second",
    "Speedtest current Upload speed in bits/s",
    labels,
)
up = Gauge("speedtest_up", "Speedtest status whether the scrape worked", labels)
info = Info("speedtest_server", "Speedtest server information", labels)

# Cache metrics for how long (seconds)?
cache_seconds = int(os.environ.get('SPEEDTEST_CACHE_FOR', 0))
cache_until = datetime.datetime.fromtimestamp(0)


def bytes_to_bits(bytes_per_sec):
    return bytes_per_sec * 8


def bits_to_megabits(bits_per_sec):
    megabits = round(bits_per_sec * (10**-6), 2)
    return str(megabits) + "Mbps"


def is_json(myjson):
    try:
        json.loads(myjson)
    except ValueError:
        return False
    return True


class Result:
    @classmethod
    def parse(cls, data):
        try:
            return cls(data)
        except ValueError:
            return None

    def __init__(self, data):
        if data["type"] == "result":
            self._data = data
        else:
            raise ValueError("data is not a result")

    def __str__(self):
        return (
            f"Server={self.server_id} "
            f"Name={self.server_name} "
            f"Latency={self.latency}ms "
            f"Jitter={self.jitter}ms "
            f"Download={bits_to_megabits(self.download_speed)} "
            f"Upload={bits_to_megabits(self.upload_speed)}"
        )

    @property
    def server_id(self):
        return int(self._data["server"]["id"])

    @property
    def server_name(self):
        return self._data["server"]["name"]

    @property
    def server_info(self):
        return {
            "id": str(self.server_id),
            "name": self.server_name,
            "location": self._data["server"]["location"],
            "country": self._data["server"]["country"],
        }

    @property
    def latency(self):
        return self._data["ping"]["latency"]

    @property
    def jitter(self):
        return self._data["ping"]["jitter"]

    @property
    def download_speed(self):
        return bytes_to_bits(self._data["download"]["bandwidth"])

    @property
    def upload_speed(self):
        return bytes_to_bits(self._data["upload"]["bandwidth"])


def execTest(server_id: int | None):
    cmd = [
        "speedtest",
        "--format=json-pretty",
        "--progress=no",
        "--accept-license",
        "--accept-gdpr",
    ]
    if server_id:
        cmd.append(f"--server-id={server_id}")
    try:
        timeout = int(os.environ.get("SPEEDTEST_TIMEOUT", 90))
        output = subprocess.check_output(cmd, timeout=timeout)
    except subprocess.CalledProcessError as e:
        output = e.output
        if not is_json(output):
            if len(output) > 0:
                logging.error(
                    "Speedtest CLI Error occurred that was not in JSON format"
                )
            return None
    except subprocess.TimeoutExpired:
        logging.error(
            "Speedtest CLI process took too long to complete and was killed."
        )
        return None

    if is_json(output):
        data = json.loads(output)
        if "error" in data:
            # Socket error
            print("Something went wrong")
            print(data["error"])
            return None
        if "type" in data:
            if data["type"] == "log":
                print(str(data["timestamp"]) + " - " + str(data["message"]))
            if data["type"] == "result":
                return Result.parse(data)


def runTest(server_id: int | None = None):
    result = execTest(server_id)
    server_id = str(result.server_id if result else server_id or "unknown")

    info.labels(server_id).info(result.server_info if result else {})
    ping.labels(server_id).set(result.latency if result else 0)
    jitter.labels(server_id).set(result.jitter if result else 0)
    download_speed.labels(server_id).set(result.download_speed if result else 0)
    upload_speed.labels(server_id).set(result.upload_speed if result else 0)
    up.labels(server_id).set(1 if result else 0)

    if result:
        logging.info(result)


@app.route("/metrics")
def updateResults():
    global cache_until

    if datetime.datetime.now() > cache_until:
        server_ids = os.environ.get("SPEEDTEST_SERVER")
        if server_ids:
            for server_id in server_ids.split(","):
                runTest(int(server_id))
        else:
            runTest()

        cache_until = datetime.datetime.now() + datetime.timedelta(
            seconds=cache_seconds
        )

    return make_wsgi_app()


@app.route("/")
def mainPage():
    return ("<h1>Welcome to Speedtest-Exporter.</h1>" +
            "Click <a href='/metrics'>here</a> to see metrics.")


def checkForBinary():
    if which("speedtest") is None:
        logging.error("Speedtest CLI binary not found. Please install it by" +
                      " going to the official website.\n" +
                      "https://www.speedtest.net/apps/cli")
        exit(1)
    speedtestVersionDialog = (subprocess.run(['speedtest', '--version'],
                              capture_output=True, text=True))
    if "Speedtest by Ookla" not in speedtestVersionDialog.stdout:
        logging.error("Speedtest CLI that is installed is not the official" +
                      " one. Please install it by going to the official" +
                      " website.\nhttps://www.speedtest.net/apps/cli")
        exit(1)


if __name__ == '__main__':
    checkForBinary()
    PORT = os.getenv('SPEEDTEST_PORT', 9798)
    logging.info("Starting Speedtest-Exporter on http://localhost:" +
                 str(PORT))
    serve(app, host='0.0.0.0', port=PORT)
