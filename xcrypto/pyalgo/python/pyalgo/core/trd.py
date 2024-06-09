from pyalgo import *
from datetime import datetime
from .base import ContextBase
from typing import Optional


class Tradable:
    """"""

    def __init__(self, subscription: Subscription, ctx: ContextBase):
        self.ctx = ctx

        self.subscription = subscription
        self.on_order = lambda x: None

    @property
    def symbol(self) -> str:
        return self.subscription.symbol

    @property
    def delivery(self) -> str:
        return self.subscription.delivery

    @property
    def onboard(self) -> str:
        return self.subscription.onboard

    @property
    def max_prc(self) -> float:
        return self.subscription.max_prc

    @property
    def min_prc(self) -> float:
        return self.subscription.min_prc

    @property
    def tick_size(self) -> float:
        return self.subscription.tick_size

    @property
    def lot(self) -> float:
        return self.subscription.lot

    @property
    def market_lot(self) -> float:
        return self.subscription.market_lot

    @property
    def min_notional(self) -> float:
        return self.subscription.min_notional

    @property
    def net(self) -> float:
        return self.subscription.net

    def order_support(self, order_type: OrderType) -> bool:
        return self.subscription.order_support(order_type)

    def tif_support(self, tif: Tif) -> bool:
        return self.subscription.tif_support(tif)

    def floor_to_lot_size(self, vol: float) -> float:
        return self.subscription.floor_to_lot_size(vol)

    def round_price(self, price: float) -> float:
        return self.subscription.round_price(price)

    def tick_up(self, price: float, n: int) -> float:
        return self.subscription.tick_up(price, n)

    def tick_dn(self, price: float, n: int) -> float:
        return self.subscription.tick_dn(price, n)

    def add_phase(self, hour: int, minute: int, second: int, phase: Phase):
        self.subscription.add_phase(hour, minute, second, phase)

    def determine(self, mills: int) -> Phase:
        return self.subscription.determine(mills)

    def add_order(
        self,
        price: float,
        quantity: float,
        side: Side,
        order_type: OrderType,
        tif: Tif,
    ) -> Optional[Order]:
        return self.ctx.add_order(self.symbol, price, quantity, side, order_type, tif)

    def cancel(self, order_id: int):
        self.ctx.cancel(self.symbol, order_id)


class DepthSubscription(Tradable):
    """"""

    def __init__(self, subscription: Subscription, ctx: ContextBase):
        super().__init__(subscription, ctx)
        self.on_data = lambda x: None
        self.data: Depth = None

    @property
    def time(self) -> int:
        return self.data.time if self.data else 0

    @property
    def datetime(self) -> datetime:
        return self.data.datetime if self.data else datetime.min

    @property
    def bid_level(self):
        return self.data.bid_level if self.data else 0

    @property
    def ask_level(self):
        return self.data.ask_level if self.data else 0

    @property
    def phase(self) -> Phase:
        return self.determine(self.time)

    def bid_prc(self, level: int) -> float:
        return self.data.bid_prc(level) if self.data else 0.0

    def bid_vol(self, level: int) -> float:
        return self.data.bid_vol(level) if self.data else 0.0

    def ask_vol(self, level: int) -> float:
        return self.data.ask_vol(level) if self.data else 0.0

    def ask_prc(self, level: int) -> float:
        return self.data.ask_prc(level) if self.data else 0.0

    def on_market(self, data: Depth):
        self.data = data
        self.on_data(data)


class BarSubscription(Tradable):
    """"""

    def __init__(self, subscription: Subscription, ctx: ContextBase):
        super().__init__(subscription, ctx)
        self.on_data = lambda x: None
        self.data: Kline = None

    @property
    def time(self) -> int:
        return self.data.time if self.data else 0

    @property
    def datetime(self) -> datetime:
        return self.data.datetime if self.data else datetime.min

    @property
    def open(self) -> float:
        return self.data.open if self.data else 0.0

    @property
    def high(self) -> float:
        return self.data.high if self.data else 0.0

    @property
    def low(self) -> float:
        return self.data.low if self.data else 0.0

    @property
    def close(self) -> float:
        return self.data.close if self.data else 0.0

    @property
    def volume(self) -> float:
        return self.data.volume if self.data else 0.0

    @property
    def amount(self) -> float:
        return self.data.amount if self.data else 0.0

    @property
    def phase(self) -> Phase:
        return self.determine(self.time)

    def on_market(self, data: Depth):
        self.data = data
        self.on_data(data)


class SmartOrder:
    """"""

    def __init__(self, subscription: Tradable):
        self.subscription = subscription
        self.order: Optional[Order] = None

    @property
    def side(self) -> Side:
        return self.order.side if self.order else Side.UNDEF

    @property
    def state(self) -> State:
        return self.order.state if self.order else State.UNDEF

    @property
    def order_type(self) -> OrderType:
        return self.order.order_type if self.order_type else OrderType.UNDEF

    @property
    def tif(self) -> Tif:
        return self.order.tif if self.order else Tif.UNDEF

    @property
    def quantity(self) -> float:
        return self.order.quantity if self.order else 0.0

    @property
    def price(self) -> float:
        return self.order.price if self.order else 0.0

    @property
    def acc(self) -> float:
        return self.order.acc if self.order else 0.0

    @property
    def making(self) -> bool:
        return self.order.making if self.order else False

    @property
    def is_active(self) -> bool:
        return self.order.is_active if self.order else False

    def send(
        self,
        price: float,
        quantity: float,
        side: Side,
        order_type: OrderType,
        tif: Tif = Tif.GTC,
    ):
        if self.is_active:
            return

        price = self.subscription.round_price(price)
        quantity = self.subscription.floor_to_lot_size(quantity)

        self.order = self.subscription.add_order(price, quantity, side, order_type, tif)

    def kill(self):
        if not self.is_active:
            return

        self.subscription.cancel(self.order.id)
