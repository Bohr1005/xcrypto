from pyalgo import init_logger
from .context import Context
from typing import Dict, Tuple
from time import sleep
import sys
import signal
from pathlib import Path


class Engine:
    """"""

    def __init__(self, interval: float, level: str = "info"):
        self.APPNAME = Path(sys.argv[0]).stem
        self.logger = init_logger(level, f"log/{self.APPNAME}")
        self.interval = interval

        self.contexts: Dict[Tuple[str, int], Context] = {}
        self.active = False

        signal.signal(signal.SIGTERM, self.stop)
        signal.signal(signal.SIGINT, self.stop)

    def make_session(
        self, addr: str, session_id: int, name: str, trading: bool = False
    ) -> Context:
        key = (addr, session_id)
        if key in self.contexts:
            return self.contexts[key]

        context = Context(addr, session_id, name, trading)
        self.contexts[key] = context

        context.connect()
        
        return context

    def run(self):
        self.active = True

        while self.active:
            for context in self.contexts.values():
                context.process()
            sleep(self.interval)

    def stop(self, *_):
        self.active = False
