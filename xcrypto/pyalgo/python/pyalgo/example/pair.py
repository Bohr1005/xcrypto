from pyalgo import *


class Demo:
    """"""

    def __init__(self, leading: DepthSubscription, hedge: DepthSubscription):
        self.leading = leading
        self.hedge = hedge

        # set callback
        self.leading.on_data = self.on_depth
        self.hedge.on_data = self.on_depth

    def on_depth(self, depth: Depth):
        print(
            depth.datetime,
            depth.symbol,
            self.leading.bid_prc(0) - self.hedge.bid_prc(0),
        )


if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    leading = session.subscribe("dogeusdt", "depth")
    hedge = session.subscribe("btcusdt", "depth")
    demo = Demo(leading, hedge)
    eng.run()
