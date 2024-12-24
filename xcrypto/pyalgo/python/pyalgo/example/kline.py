from pyalgo import *


class Demo:
    """"""

    def __init__(self, depth: DepthSubscription, kline: BarSubscription):
        self.depth = depth
        self.kline = kline

        # set callback
        self.depth.on_data = self.on_depth
        self.kline.on_data = self.on_kline

    def on_kline(self, kline: Kline):
        print(
            self.kline.datetime,
            self.kline.open,
            self.kline.high,
            self.kline.low,
            self.kline.close,
        )

    def on_depth(self, depth: Depth):
        print(
            depth.datetime,
            depth.symbol,
        )


if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    depth = session.subscribe("dogeusdt", "depth")
    kline = session.subscribe("btcusdt", "kline:1m")
    demo = Demo(depth, kline)
    eng.run()
