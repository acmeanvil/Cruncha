# SPDX-License-Identifier: GPL-3.0-or-later
"""The worker without the app: dependency scan, values, running cells, and the protocol.

    <python> -m unittest discover -s python/tests -t python
"""

import io
import os
import subprocess
import sys
import unittest
from fractions import Fraction

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, HERE)

from cruncha_worker import main, protocol, values  # noqa: E402
from cruncha_worker.scan import scan  # noqa: E402


def num(v, unit=None):
    if isinstance(v, Fraction) or isinstance(v, int):
        f = Fraction(v)
        value = {"type": "exact", "num": str(f.numerator), "den": str(f.denominator)}
    else:
        value = {"type": "float", "value": v}
    return {"type": "number", "value": value, "unit": unit}


KIP_PER_FT = {"name": "kip/ft", "factor": 14593.902937206364, "gauge": 0.0, "dims": [0, 1, -2, 0, 0, 0, 0, 0]}
FT = {"name": "ft", "factor": 0.3048, "gauge": 0.0, "dims": [1, 0, 0, 0, 0, 0, 0, 0]}


class Scan(unittest.TestCase):
    def test_defines_reads_imports_and_privates(self):
        r = scan("import numpy as np\nx = np.linspace(0, L, 50)\nmoment = w * x * (L - x) / 2\n_tmp = 1")
        self.assertEqual(r["defines"], ["x", "moment"])
        self.assertEqual(r["reads"], ["L", "w"])
        self.assertEqual(r["warnings"], [])

    def test_functions_classes_loops_and_nested_reads(self):
        src = (
            "def f(a):\n    return a * k + g(a)\n"
            "class C:\n    size = n\n"
            "for i in range(3):\n    total = i\n"
            "with open(path) as fh:\n    text = fh.read()\n"
            "ys = [f(v) for v in xs]\n"
            "if (m := max(xs)) > 0:\n    big = [y for y in ys if (last := y)]\n"
        )
        r = scan(src)
        self.assertEqual(r["defines"], ["f", "C", "i", "total", "fh", "text", "ys", "m", "big", "last"])
        self.assertEqual(r["reads"], ["g", "k", "n", "path", "xs"])

    def test_self_reference_is_a_define_and_builtins_are_not_reads(self):
        r = scan("x = len(items)\nx += offset\nprint(x)")
        self.assertEqual(r["defines"], ["x"])
        self.assertEqual(r["reads"], ["items", "offset"])

    def test_deleted_names_and_exception_names_are_not_defines(self):
        r = scan("for k, v in pairs:\n    total = v\ndel k, v\ntry:\n    x = 1\nexcept Exception as e:\n    pass")
        self.assertEqual(r["defines"], ["total", "x"])

    def test_dynamic_code_warns_and_declarations_override(self):
        self.assertTrue(scan("from math import *\ny = sin(1)")["warnings"])
        self.assertTrue(scan("exec('z = 1')")["warnings"])
        r = scan("# cruncha: defines z\n# cruncha: reads q\nexec('z = q')")
        self.assertEqual((r["defines"], r["reads"], r["warnings"]), (["z"], ["q"], []))

    def test_syntax_error(self):
        r = scan("x = (1,")
        self.assertEqual(r["error"]["type"], "SyntaxError")
        self.assertEqual(r["error"]["line"], 1)


class Values(unittest.TestCase):
    def test_numbers(self):
        self.assertEqual(values.from_plain(num(Fraction(3, 4))), Fraction(3, 4))
        self.assertEqual(values.from_plain(num(5)), 5)
        self.assertEqual(values.to_plain(Fraction(1, 3)), num(Fraction(1, 3)))
        self.assertEqual(values.to_plain(2.5), num(2.5))
        self.assertEqual(values.to_plain(True), {"type": "bool", "value": True})

    def test_quantities_keep_their_unit(self):
        w = values.from_plain(num(1.5, KIP_PER_FT))
        L = values.from_plain(num(26, FT))
        self.assertEqual(f"{w.units:~P}", "kip/ft")
        M = (w * L**2 / 8).to("kip*ft")
        p = values.to_plain(M)
        self.assertAlmostEqual(p["value"]["value"], 126.75)
        self.assertEqual(p["unit"]["name"], "ft·kip")
        self.assertAlmostEqual(p["unit"]["factor"], 1355.8179483314, places=6)
        self.assertEqual(p["unit"]["dims"], [2, 1, -2, 0, 0, 0, 0, 0])

    def test_unknown_unit_names_fall_back_to_si(self):
        odd = {"name": "furlong-ish", "factor": 0.3048, "gauge": 0.0, "dims": [1, 0, 0, 0, 0, 0, 0, 0]}
        q = values.from_plain(num(10.0, odd))
        self.assertAlmostEqual(q.to("meter").magnitude, 3.048)

    def test_temperature_gauge(self):
        c = {"name": "°C", "factor": 1.0, "gauge": 273.15, "dims": [0, 0, 0, 0, 1, 0, 0, 0]}
        t = values.from_plain(num(20.0, c))
        self.assertAlmostEqual(t.to("kelvin").magnitude, 293.15)
        p = values.to_plain(t)
        self.assertAlmostEqual(p["unit"]["gauge"], 273.15)
        self.assertEqual(p["unit"]["name"], "°C")

    def test_arrays(self):
        a = values.from_plain({"type": "array", "shape": [2, 2], "orientation": "matrix", "values": [1, 2, 3, 4], "unit": FT})
        self.assertEqual(a.magnitude.shape, (2, 2))
        p = values.to_plain(a)
        self.assertEqual((p["shape"], p["values"], p["unit"]["name"]), ([2, 2], [1, 2, 3, 4], "ft"))
        import numpy as np

        self.assertEqual(values.to_plain(np.zeros(200_000))["type"], "unsupported")
        with self.assertRaises(ValueError):
            values.from_plain({"type": "array", "shape": [3], "orientation": "column", "values": [1, 2], "unit": None})

    def test_frames_and_unsupported(self):
        import polars as pl

        p = values.to_plain(pl.DataFrame({"a": [1, 2], "b": ["x", None]}))
        self.assertEqual(p["columns"][0]["data"], {"type": "number", "values": [1.0, 2.0]})
        self.assertEqual(p["columns"][1]["data"], {"type": "text", "values": ["x", None]})
        back = values.from_plain(p)
        self.assertEqual(back.shape, (2, 2))
        self.assertEqual(values.to_plain(lambda: 1)["type"], "unsupported")


class Run(unittest.TestCase):
    def setUp(self):
        main.STORE.clear()

    def test_inputs_defines_stdout_and_last_expression(self):
        r = main.run(
            {"cell": "A", "source": "print('hi')\nM = w * L**2 / 8\nM.to('kip*ft')", "defines": ["M"],
             "inputs": {"w": num(1.5, KIP_PER_FT), "L": num(26, FT)}},
            [],
        )
        self.assertEqual(r["status"], "done", r["error"])
        self.assertEqual(r["stdout"], "hi\n")
        self.assertEqual(r["defines"][0]["name"], "M")
        self.assertIn("126.75", r["outputs"][0]["text"])
        # A later cell reads M from the worker, not from the app.
        r2 = main.run({"cell": "B", "source": "M2 = 2 * M", "defines": ["M2"], "python_inputs": ["M"]}, [])
        self.assertEqual(r2["status"], "done", r2["error"])
        self.assertIn("M2", main.STORE)

    def test_cells_see_only_what_they_read(self):
        main.run({"cell": "A", "source": "secret = 1", "defines": ["secret"]}, [])
        r = main.run({"cell": "B", "source": "y = secret", "defines": ["y"]}, [])
        self.assertEqual(r["status"], "error")
        self.assertEqual(r["error"]["type"], "NameError")

    def test_errors_point_at_the_cell_line(self):
        r = main.run({"cell": "A", "source": "a = 1\nb = a / 0\n", "defines": ["a", "b"]}, [])
        self.assertEqual(r["status"], "error")
        self.assertEqual((r["error"]["type"], r["error"]["line"]), ("ZeroDivisionError", 2))
        self.assertIn("line 2: b = a / 0", r["error"]["traceback"])
        self.assertNotIn("a", main.STORE, "a failed cell defines nothing")
        r = main.run({"cell": "A", "source": "raise SystemExit(3)", "defines": []}, [])
        self.assertEqual(r["error"]["type"], "SystemExit")

    def test_figures_and_tables(self):
        blobs = []
        r = main.run({"cell": "P", "source": "import matplotlib.pyplot as plt\nplt.plot([1, 2, 3])\nimport polars as pl\npl.DataFrame({'a': range(100)})", "defines": []}, blobs)
        self.assertEqual(r["status"], "done", r["error"])
        kinds = [o["kind"] for o in r["outputs"]]
        self.assertEqual(kinds, ["image", "table"])
        self.assertTrue(blobs[0].startswith(b"\x89PNG"))
        self.assertEqual((len(r["outputs"][1]["rows"]), r["outputs"][1]["total_rows"]), (50, 100))


class Protocol(unittest.TestCase):
    def test_framing_round_trip(self):
        buf = io.BytesIO()
        protocol.write_message(buf, {"op": "x", "n": 1}, [b"abc", b""])
        buf.seek(0)
        h, blobs = protocol.read_message(buf)
        self.assertEqual((h["op"], h["blobs"], blobs), ("x", 2, [b"abc", b""]))

    def test_worker_process(self):
        p = subprocess.Popen([sys.executable, "-E", "-s", "-m", "cruncha_worker"], cwd=HERE,
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        protocol.write_message(p.stdin, {"id": 1, "op": "hello"})
        h, _ = protocol.read_message(p.stdout)
        self.assertTrue(h["ok"])
        # A cell printing to the real fd 1 (as C code would) can't corrupt the channel.
        protocol.write_message(p.stdin, {"id": 2, "op": "run", "cell": "X", "source": "import os\nos.write(1, b'noise')\nv = 7", "defines": ["v"]})
        h, _ = protocol.read_message(p.stdout)
        self.assertEqual((h["id"], h["status"]), (2, "done"))
        protocol.write_message(p.stdin, {"id": 3, "op": "shutdown"})
        protocol.read_message(p.stdout)
        p.wait(5)
        self.assertIn(b"noise", p.stderr.read())
        for f in (p.stdin, p.stdout, p.stderr):
            f.close()


if __name__ == "__main__":
    unittest.main()
