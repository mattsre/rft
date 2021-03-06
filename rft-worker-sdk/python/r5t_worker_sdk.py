"""A Python SDK for using rft to process work in parallel"""
import json


__version__ = "0.1.0"


def job_id(datapath="/input/data.json"):
    try:
        with open(datapath) as read_file:
            job_config = json.load(read_file)

            return job_config["job_id"]
    except FileNotFoundError as err:
        print("Provided datapath did not contain an rft datafile")
        raise


def get_param(str, fallback, datapath="/input/data.json"):
    try:
        with open(datapath) as read_file:
            job_config = json.load(read_file)

            return job_config["params"][str]
    except FileNotFoundError:
        print("No rft datafile found, using fallback value")
        return fallback
