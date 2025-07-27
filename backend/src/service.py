from zeroconf import ServiceInfo, Zeroconf
import socket
import uuid
from server import SERVER_PORT

SERVICE_TYPE = "_shared-cb._tcp.local."


def register_service(zeroconf: Zeroconf, host: str):
    hostname = socket.gethostname()
    service_name = f"{hostname}-{uuid.uuid4().hex[:6]}.{SERVICE_TYPE}"

    ip_bytes = socket.inet_aton(host)

    info = ServiceInfo(
        type_=SERVICE_TYPE,
        name=service_name,
        addresses=[ip_bytes],
        port=SERVER_PORT,
    )

    print(f"Advertising {service_name} on {host}:{SERVER_PORT}")
    zeroconf.register_service(info)

    return info
