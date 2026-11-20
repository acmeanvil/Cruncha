# SPDX-License-Identifier: GPL-3.0-or-later
"""What Python cells import from Cruncha.

``ureg`` is the unit registry that calc values arrive in, so quantities from calcs and from
Python cells combine::

    from cruncha import ureg, Q_
    F = Q_(12, "kip")
    stress = F / A          # A came from a calc cell, e.g. `A = 3.5 in²`

Only quantities built with this registry can go back to calc cells with their units.
"""

__all__ = ["ureg", "Q_", "Quantity"]

_ureg = None


def _registry():
    global _ureg
    if _ureg is None:
        import pint

        ureg = pint.UnitRegistry(autoconvert_offset_to_baseunit=True)
        # Units Hurmet knows that pint's default registry lacks (or names differently).
        for line in (
            "EUR = [currency]",
            "ksf = kip / foot ** 2",
            "klf = kip / foot",
            "plf = force_pound / foot",
            "psf = force_pound / foot ** 2",
        ):
            try:
                ureg.define(line)
            except Exception:  # already defined in this pint version
                pass
        pint.set_application_registry(ureg)
        _ureg = ureg
    return _ureg


def __getattr__(name):
    if name == "ureg":
        return _registry()
    if name in ("Q_", "Quantity"):
        return _registry().Quantity
    raise AttributeError(name)
