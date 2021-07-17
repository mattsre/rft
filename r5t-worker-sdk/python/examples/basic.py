import sys
import os

PACKAGE_PARENT = ".."
SCRIPT_DIR = os.path.dirname(
    os.path.realpath(os.path.join(os.getcwd(), os.path.expanduser(__file__)))
)
sys.path.append(os.path.normpath(os.path.join(SCRIPT_DIR, PACKAGE_PARENT)))

import r5t_worker_sdk as r5t

print(r5t.job_id("../example-data.json"))
print(r5t.get_param("start_date", 2000, "../example-data.json"))
