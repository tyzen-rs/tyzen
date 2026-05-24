#!/usr/bin/env bash
set -euo pipefail

RUNS="${1:-6}"
WARMUP="${2:-1}"
RESULT_FILE="tyzen/bench/results/build-time-delta-latest.json"

mkdir -p tyzen/bench/results

measure_check_ms() {
  local example="$1"
  local start end
  start=$(python3 - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
  cargo check -q -p tyzen --example "$example" >/dev/null
  end=$(python3 - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
  python3 - <<PY
start = int(${start})
end = int(${end})
print((end - start) / 1_000_000)
PY
}

base_samples=()
tyzen_samples=()

for i in $(seq 1 "$WARMUP"); do
  measure_check_ms "cold_baseline" >/dev/null
  measure_check_ms "cold_generate_fixture" >/dev/null
done

for i in $(seq 1 "$RUNS"); do
  b=$(measure_check_ms "cold_baseline")
  t=$(measure_check_ms "cold_generate_fixture")
  base_samples+=("$b")
  tyzen_samples+=("$t")
done

json_base=$(python3 - "${base_samples[@]}" <<'PY'
import json
import sys

vals = [float(x) for x in sys.argv[1:]]
print(json.dumps(vals))
PY
)

json_tyzen=$(python3 - "${tyzen_samples[@]}" <<'PY'
import json
import sys

vals = [float(x) for x in sys.argv[1:]]
print(json.dumps(vals))
PY
)

stats=$(python3 - <<PY
import json, math
b = sorted(json.loads('''${json_base}'''))
t = sorted(json.loads('''${json_tyzen}'''))
median_b = b[len(b)//2]
median_t = t[len(t)//2]
delta = median_t - median_b
ratio = (median_t / median_b) if median_b else 0.0
print(f"{median_b:.3f} {median_t:.3f} {delta:.3f} {ratio:.3f}")
PY
)

median_base=$(echo "$stats" | awk '{print $1}')
median_tyzen=$(echo "$stats" | awk '{print $2}')
delta_ms=$(echo "$stats" | awk '{print $3}')
ratio=$(echo "$stats" | awk '{print $4}')

cat > "$RESULT_FILE" <<JSON
{
  "benchmark": "build-time-delta-check",
  "warmup_runs": ${WARMUP},
  "measured_runs": ${RUNS},
  "baseline_example": "cold_baseline",
  "tyzen_example": "cold_generate_fixture",
  "baseline_ms_median": ${median_base},
  "tyzen_ms_median": ${median_tyzen},
  "delta_ms_median": ${delta_ms},
  "ratio_median": ${ratio},
  "baseline_samples_ms": ${json_base},
  "tyzen_samples_ms": ${json_tyzen}
}
JSON

echo "benchmark=build-time-delta-check baseline_median_ms=${median_base} tyzen_median_ms=${median_tyzen} delta_ms=${delta_ms} ratio=${ratio} result=${RESULT_FILE}"
