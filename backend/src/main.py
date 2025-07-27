from zeroconf import Zeroconf
from listener import start_listener
from service import register_service
from server import start_server
from firewall import update_firewall
import socket
import threading
import time


def main():
    # # get os allocated port number
    # sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    # sock.bind(("", 0))
    # port = sock.getsockname()[1]
    # sock.close()

    # get lan ip address
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.connect(("8.8.8.8", 80))
    ip = s.getsockname()[0]
    s.close()

    zeroconf = Zeroconf()

    update_firewall()

    own_info = register_service(zeroconf, ip)
    start_listener(zeroconf, own_info)

    threading.Thread(target=start_server, args=(ip,), daemon=True).start()

    try:
        while True:
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\n[Main] Exiting")
    finally:
        zeroconf.unregister_service(own_info)
        zeroconf.close()


if __name__ == "__main__":
    main()
