from pyalgo import *
from typing import Union, Dict
from .trd import *


class Context(ContextBase):
    """"""

    def __init__(self, addr: str, session_id: int, name: str, trading: bool):
        self.session = Session(addr, session_id, name, trading)

        self.tradings: Dict[str, Tradable] = {}
        self.subscriptions: Dict[str, Union[DepthSubscription, BarSubscription]] = {}

    @property
    def id(self):
        return self.session.id

    @property
    def name(self):
        return self.session.name

    @property
    def trading(self):
        return self.session.trading

    @property
    def is_login(self):
        return self.session.is_login

    def connect(self):
        self.session.connect()

    def on_market(self, data: Union[Depth, Kline]):
        if sub := self.subscriptions.get(data.stream):
            sub.on_market(data)

    def on_order(self, order: Order):
        if trading := self.tradings.get(order.symbol):
            trading.on_order(order)

    def subscribe(
        self, symbol: str, stream: str
    ) -> Union[DepthSubscription, BarSubscription]:
        if symbol in self.tradings:
            raise Exception(f"Duplicate subscribe {symbol}")

        sub = self.session.subscribe(symbol, stream)

        key = symbol + "@" + stream

        if stream.startswith("kline"):
            bar = BarSubscription(sub, self)
            self.subscriptions[key] = bar
            self.tradings[symbol] = bar

            return bar

        match stream:
            case "depth" | "bbo":
                depth = DepthSubscription(sub, self)
                self.subscriptions[key] = depth
                self.tradings[symbol] = depth

                return depth

            case _:
                raise Exception(f"Unsupported stream {stream}")

    def process(self):
        if event := self.session.process():
            match event.event_type:
                case EventType.Depth | EventType.Kline:
                    self.on_market(event.data)

                case EventType.Order:
                    self.on_order(event.data)

            return event

    def add_order(
        self,
        symbol: str,
        price: float,
        quantity: float,
        side: Side,
        order_type: OrderType,
        tif: Tif,
    ) -> Optional[Order]:
        return self.session.add_order(symbol, price, quantity, side, order_type, tif)

    def cancel(self, symbol: str, order_id: int):
        self.session.cancel(symbol, order_id)
