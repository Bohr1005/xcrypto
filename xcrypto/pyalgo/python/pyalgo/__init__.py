import sys
from .pyalgo import *
from pyalgo.core.trd import SmartOrder, DepthSubscription, BarSubscription
from pyalgo.core.engine import Engine
from pyalgo.core.context import Context


def info(msg: str):
    frame = sys._getframe(1)
    file = frame.f_code.co_filename
    lineno = frame.f_lineno
    pyalgo.log_info(file, lineno, msg)


def debug(msg: str):
    frame = sys._getframe(1)
    file = frame.f_code.co_filename
    lineno = frame.f_lineno
    pyalgo.log_debug(file, lineno, msg)


def warn(msg: str):
    frame = sys._getframe(1)
    file = frame.f_code.co_filename
    lineno = frame.f_lineno
    pyalgo.log_warn(file, lineno, msg)


def error(msg: str):
    frame = sys._getframe(1)
    file = frame.f_code.co_filename
    lineno = frame.f_lineno
    pyalgo.log_error(file, lineno, msg)


__all__ = [
    "info",
    "debug",
    "warn",
    "error",
    "DepthSubscription",
    "BarSubscription",
    "Engine",
    "Context",
    "SmartOrder",
]

__doc__ = pyalgo.__doc__
if hasattr(pyalgo, "__all__"):
    __all__.extend(pyalgo.__all__)
