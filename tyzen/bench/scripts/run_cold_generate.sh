#!/usr/bin/env bash
set -euo pipefail

RUNS="${1:-10}"
WARMUP="${2:-2}"
RESULT_FILE="tyzen/bench/results/cold-generate-latest.json"

mkdir -p tyzen/bench/results target/bench-bindings

measure_ms() {
  local out_file="$1"
  local start end
  start=$(python3 - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
  cargo run -q -p tyzen --release --example cold_generate_fixture -- "$out_file" >/dev/null
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

samples=()

for i in $(seq 1 "$WARMUP"); do
  measure_ms "target/bench-bindings/cold-warmup-${i}.ts" >/dev/null
  rm -f "target/bench-bindings/cold-warmup-${i}.ts"
done

for i in $(seq 1 "$RUNS"); do
  ms=$(measure_ms "target/bench-bindings/cold-run-${i}.ts")
  samples+=("$ms")
  rm -f "target/bench-bindings/cold-run-${i}.ts"
done

json_samples=$(python3 - "${samples[@]}" <<'PY'
import json
import sys

vals = [float(x) for x in sys.argv[1:]]
print(json.dumps(vals))
PY
)

stats=$(python3 - <<PY
import json, statistics
vals = json.loads('''${json_samples}''')
vals_sorted = sorted(vals)
mid = len(vals_sorted)//2
median = vals_sorted[mid]
idx95 = max(0, int(__import__('math').ceil(len(vals_sorted)*0.95)-1))
p95 = vals_sorted[idx95]
print(f"{median:.3f} {p95:.3f}")
PY
)

median=$(echo "$stats" | awk '{print $1}')
p95=$(echo "$stats" | awk '{print $2}')

cat > "$RESULT_FILE" <<JSON
{
  "benchmark": "cold-process-generate",
  "warmup_runs": ${WARMUP},
  "measured_runs": ${RUNS},
  "total_ms_median": ${median},
  "total_ms_p95": ${p95},
  "samples_ms": ${json_samples}
}
JSON

echo "benchmark=cold-process-generate total_median_ms=${median} total_p95_ms=${p95} result=${RESULT_FILE}"
