from flask import Flask

app = Flask(__name__)

SERVER_PORT = 7001


@app.route("/")
def hello_world():
    print()
    return "HI"


def start_server(host: str):
    app.run(host, SERVER_PORT)
