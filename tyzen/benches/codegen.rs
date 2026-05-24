use std::hint::black_box;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tyzen::generator::GeneratorConfig;
use tyzen::meta::{FieldMeta, StructMeta, TypeStructure};
use tyzen::registry::TypeMeta;
use tyzen::renderer::render_type;

const WARMUP_RUNS: usize = 5;
const MEASURED_RUNS: usize = 30;

const SMALL_TYPE_COUNT: usize = 20;
const SMALL_FIELD_COUNT: usize = 10;

const MEDIUM_TYPE_COUNT: usize = 200;
const MEDIUM_FIELD_COUNT: usize = 16;

const LARGE_TYPE_COUNT: usize = 1000;
const LARGE_FIELD_COUNT: usize = 24;

static SMALL_FIELDS: OnceLock<&'static [FieldMeta]> = OnceLock::new();
static MEDIUM_FIELDS: OnceLock<&'static [FieldMeta]> = OnceLock::new();
static LARGE_FIELDS: OnceLock<&'static [FieldMeta]> = OnceLock::new();

fn ts_string() -> String {
    "string".to_string()
}

fn build_fields(field_count: usize) -> &'static [FieldMeta] {
    let mut fields = Vec::with_capacity(field_count);
    for i in 0..field_count {
        let name: &'static str = Box::leak(format!("f{i}").into_boxed_str());
        fields.push(FieldMeta {
            name,
            ty_name: ts_string,
            optional: false,
            flattened: false,
            flatten_base_name: None,
            is_binary: false,
            validation: None,
            nullable: false,
        });
    }
    Box::leak(fields.into_boxed_slice())
}

fn structure_small() -> TypeStructure {
    let fields = SMALL_FIELDS.get_or_init(|| build_fields(SMALL_FIELD_COUNT));
    TypeStructure::Struct(StructMeta { fields })
}

fn structure_medium() -> TypeStructure {
    let fields = MEDIUM_FIELDS.get_or_init(|| build_fields(MEDIUM_FIELD_COUNT));
    TypeStructure::Struct(StructMeta { fields })
}

fn structure_large() -> TypeStructure {
    let fields = LARGE_FIELDS.get_or_init(|| build_fields(LARGE_FIELD_COUNT));
    TypeStructure::Struct(StructMeta { fields })
}

fn build_type_metas(type_count: usize, structure: fn() -> TypeStructure) -> Vec<TypeMeta> {
    let mut metas = Vec::with_capacity(type_count);

    for i in 0..type_count {
        let name: &'static str = Box::leak(format!("BenchType{i}").into_boxed_str());
        metas.push(TypeMeta {
            name,
            generic_params: "",
            structure,
            ns: None,
            module_path: "bench",
            has_binary: false,
            schema: false,
        });
    }

    metas
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    type_count: usize,
    field_count: usize,
    structure: fn() -> TypeStructure,
}

struct ScenarioResult {
    name: &'static str,
    type_count: usize,
    field_count: usize,
    metadata_build_ms: f64,
    render_only_ms: f64,
    total_ms: f64,
    render_p95_ms: f64,
    total_p95_ms: f64,
}

fn duration_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn percentile_95_ms(samples: &mut [Duration]) -> f64 {
    samples.sort_unstable();
    let idx = ((samples.len() as f64) * 0.95).ceil() as usize - 1;
    duration_ms(samples[idx])
}

fn median_ms(samples: &mut [Duration]) -> f64 {
    samples.sort_unstable();
    let mid = samples.len() / 2;
    duration_ms(samples[mid])
}

fn run_once(s: Scenario) -> (Duration, Duration, Duration) {
    let total_start = Instant::now();

    let build_start = Instant::now();
    let metas = build_type_metas(s.type_count, s.structure);
    let build_dur = build_start.elapsed();

    let refs: Vec<&TypeMeta> = metas.iter().collect();
    let config = GeneratorConfig::default();

    let render_start = Instant::now();
    let mut out = String::new();
    for meta in &refs {
        out.push_str(&render_type(meta, &refs, config));
        out.push('\n');
    }
    black_box(out);
    let render_dur = render_start.elapsed();

    let total_dur = total_start.elapsed();
    (build_dur, render_dur, total_dur)
}

fn run_scenario(s: Scenario) -> ScenarioResult {
    for _ in 0..WARMUP_RUNS {
        let _ = run_once(s);
    }

    let mut build_samples = Vec::with_capacity(MEASURED_RUNS);
    let mut render_samples = Vec::with_capacity(MEASURED_RUNS);
    let mut total_samples = Vec::with_capacity(MEASURED_RUNS);

    for _ in 0..MEASURED_RUNS {
        let (build_dur, render_dur, total_dur) = run_once(s);
        build_samples.push(build_dur);
        render_samples.push(render_dur);
        total_samples.push(total_dur);
    }

    let mut build_for_median = build_samples.clone();
    let mut render_for_median = render_samples.clone();
    let mut total_for_median = total_samples.clone();

    let mut render_for_p95 = render_samples;
    let mut total_for_p95 = total_samples;

    ScenarioResult {
        name: s.name,
        type_count: s.type_count,
        field_count: s.field_count,
        metadata_build_ms: median_ms(&mut build_for_median),
        render_only_ms: median_ms(&mut render_for_median),
        total_ms: median_ms(&mut total_for_median),
        render_p95_ms: percentile_95_ms(&mut render_for_p95),
        total_p95_ms: percentile_95_ms(&mut total_for_p95),
    }
}

fn write_results_json(results: &[ScenarioResult]) -> std::io::Result<()> {
    std::fs::create_dir_all("bench/results")?;

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!(
        "  \"warmup_runs\": {},\n  \"measured_runs\": {},\n  \"scenarios\": [\n",
        WARMUP_RUNS, MEASURED_RUNS
    ));

    for (idx, r) in results.iter().enumerate() {
        let comma = if idx + 1 == results.len() { "" } else { "," };
        json.push_str("    {\n");
        json.push_str(&format!("      \"name\": \"{}\",\n", r.name));
        json.push_str(&format!("      \"type_count\": {},\n", r.type_count));
        json.push_str(&format!("      \"field_count\": {},\n", r.field_count));
        json.push_str(&format!(
            "      \"metadata_build_ms_median\": {:.3},\n",
            r.metadata_build_ms
        ));
        json.push_str(&format!(
            "      \"render_only_ms_median\": {:.3},\n",
            r.render_only_ms
        ));
        json.push_str(&format!(
            "      \"render_only_ms_p95\": {:.3},\n",
            r.render_p95_ms
        ));
        json.push_str(&format!("      \"total_ms_median\": {:.3},\n", r.total_ms));
        json.push_str(&format!("      \"total_ms_p95\": {:.3}\n", r.total_p95_ms));
        json.push_str(&format!("    }}{}\n", comma));
    }

    json.push_str("  ]\n}\n");
    std::fs::write("bench/results/latest.json", json)
}

fn main() {
    let scenarios = [
        Scenario {
            name: "small",
            type_count: SMALL_TYPE_COUNT,
            field_count: SMALL_FIELD_COUNT,
            structure: structure_small,
        },
        Scenario {
            name: "medium",
            type_count: MEDIUM_TYPE_COUNT,
            field_count: MEDIUM_FIELD_COUNT,
            structure: structure_medium,
        },
        Scenario {
            name: "large",
            type_count: LARGE_TYPE_COUNT,
            field_count: LARGE_FIELD_COUNT,
            structure: structure_large,
        },
    ];

    let mut results = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        results.push(run_scenario(scenario));
    }

    write_results_json(&results).expect("failed to write benchmark results");

    for r in results {
        println!(
            "scenario={} types={} fields={} build_median_ms={:.3} render_median_ms={:.3} total_median_ms={:.3}",
            r.name, r.type_count, r.field_count, r.metadata_build_ms, r.render_only_ms, r.total_ms
        );
    }
}
