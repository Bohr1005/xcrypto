from abc import ABC
from pyalgo import Side, OrderType, Tif


# avoid circular import
class ContextBase(ABC):
    """"""

    def add_order(
        self,
        symbol: str,
        price: float,
        quantity: float,
        side: Side,
        order_type: OrderType,
        tif: Tif,
    ):
        raise NotImplemented

    def cancel(self, symbol: str, order_id: int):
        raise NotImplemented
