#!/usr/bin/env bash
# Sample memory/CPU/thread metrics for a GUI process over a fixed window.
#
# Usage:
#   measure.sh LABEL SECONDS -- COMMAND [ARGS...]   # launch and measure
#   measure.sh LABEL SECONDS PID                    # measure an existing pid
#
# Metrics: VmRSS, PSS, Private_Dirty (smaps_rollup), thread count and CPU %.
# PSS/private-dirty come from /proc/<pid>/smaps_rollup when available.
set -u

label="${1:?label required}"
seconds="${2:?seconds required}"
out="${LAMPCORD_BENCH_OUT:-/tmp/opencode/bench-${label}.log}"

if [[ "${3:-}" == "--" ]]; then
    shift 3
    "$@" >"/tmp/opencode/bench-${label}.stdout" 2>&1 &
    pid=$!
else
    pid="${3:?pid or -- command required}"
fi

warmup="${LAMPCORD_BENCH_WARMUP:-5}"
sleep "$warmup"
if ! kill -0 "$pid" 2>/dev/null; then
    echo "process $pid exited during warmup" >&2
    exit 1
fi

read_gauge() {
    local key="$1"
    awk -v k="$key" '$1 == k ":" {print $2; exit}' "/proc/$pid/status" 2>/dev/null || echo 0
}

read_rollup() {
    local key="$1"
    awk -v k="$key:" '$1 == k {print $2; exit}' "/proc/$pid/smaps_rollup" 2>/dev/null || echo 0
}

read_utime() {
    awk '{print $14 + $15}' "/proc/$pid/stat" 2>/dev/null || echo 0
}

hz=$(getconf CLK_TCK)
prev_time=$(read_utime)
prev_epoch=$(date +%s.%N)

sum_rss=0; sum_pss=0; sum_dirty=0; sum_threads=0; sum_cpu=0; samples=0
while (( samples < seconds )); do
    sleep 1
    [[ -r "/proc/$pid/status" ]] || break
    rss=$(read_gauge VmRSS)
    threads=$(read_gauge Threads)
    pss=$(read_rollup Pss)
    dirty=$(read_rollup Private_Dirty)
    now=$(read_utime)
    epoch=$(date +%s.%N)
    cpu=$(awk -v d="$((now - prev_time))" -v h="$hz" -v e="$epoch" -v p="$prev_epoch" \
        'BEGIN { t = e - p; if (t > 0) printf "%.2f", (d / h) / t * 100; else print 0 }')
    sum_rss=$((sum_rss + rss))
    sum_pss=$((sum_pss + pss))
    sum_dirty=$((sum_dirty + dirty))
    sum_threads=$((sum_threads + threads))
    sum_cpu=$(awk -v a="$sum_cpu" -v b="$cpu" 'BEGIN {printf "%.2f", a + b}')
    prev_time=$now; prev_epoch=$epoch
    samples=$((samples + 1))
done

avg() { awk -v a="$1" -v n="$samples" 'BEGIN {printf "%.1f", (n > 0 ? a / n : 0)}'; }

{
    echo "label=$label"
    echo "pid=$pid"
    echo "samples=$samples"
    echo "rss_kb_avg=$(avg "$sum_rss")"
    echo "pss_kb_avg=$(avg "$sum_pss")"
    echo "private_dirty_kb_avg=$(avg "$sum_dirty")"
    echo "threads_avg=$(avg "$sum_threads")"
    echo "cpu_pct_avg=$(awk -v a="$sum_cpu" -v n="$samples" 'BEGIN {printf "%.2f", (n > 0 ? a / n : 0)}')"
} | tee "$out"
