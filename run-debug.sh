#!/bin/bash
systemfd --no-pid -s http::5000 -- cargo watch -x run
