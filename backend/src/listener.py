from zeroconf import ServiceBrowser, ServiceListener, Zeroconf, ServiceInfo
from service import SERVICE_TYPE
import requests


def start_listener(zeroconf: Zeroconf, own_info: ServiceInfo):
    class Listener(ServiceListener):
        def update_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            print(f"Service {name} updated")

        def remove_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            print(f"Service {name} removed")

        def add_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            info = zc.get_service_info(type_, name)

            if info and info.name != own_info.name:
                service_ip = info.parsed_addresses()[0]
                service_port = info.port

                print(
                    f"Service {name} added, service info: {service_ip}, {service_port}"
                )
                response = requests.get(f"http://{service_ip}:{service_port}")
                print(response.text)

    listener = Listener()
    ServiceBrowser(zeroconf, SERVICE_TYPE, listener)
