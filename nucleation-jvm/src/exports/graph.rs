//! Redstone-graph analysis JNI exports (feature-gated with `simulation`).
//!
//! Exposes the extracted `RedstoneGraph` (via `MchprsWorld.exportGraph()`)
//! with the full analysis surface: structure queries, cycle/SCC analysis,
//! critical path, fingerprints, and JSON serialization. Aggregate views
//! (nodes, edges, features, SCCs, kind counts) are returned as JSON strings
//! so JVM consumers can use whatever JSON library they already ship.

#![cfg(feature = "simulation")]

use crate::conv::{jstring_to_string, string_to_jstring};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::simulation::fingerprint::GraphFingerprintSpec;
use nucleation::simulation::{generate_truth_table, MchprsWorld, RedstoneGraph};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nMchprsExportGraph", "(J)J", n_export_graph as *mut _),
        nm(
            "nMchprsExportGraphStructural",
            "(J)J",
            n_export_graph_structural as *mut _,
        ),
        nm("nGraphFree", "(J)V", n_graph_free as *mut _),
        nm("nGraphNodeCount", "(J)I", n_graph_node_count as *mut _),
        nm("nGraphEdgeCount", "(J)I", n_graph_edge_count as *mut _),
        nm("nGraphToJson", "(J)Ljava/lang/String;", n_graph_to_json as *mut _),
        nm("nGraphFromJson", "(Ljava/lang/String;)J", n_graph_from_json as *mut _),
        nm("nGraphNodesJson", "(J)Ljava/lang/String;", n_graph_nodes_json as *mut _),
        nm("nGraphEdgesJson", "(J)Ljava/lang/String;", n_graph_edges_json as *mut _),
        nm(
            "nGraphNodeKindCountsJson",
            "(J)Ljava/lang/String;",
            n_graph_node_kind_counts_json as *mut _,
        ),
        nm("nGraphHasCycles", "(J)Z", n_graph_has_cycles as *mut _),
        nm("nGraphIsCombinational", "(J)Z", n_graph_is_combinational as *mut _),
        nm(
            "nGraphSccsJson",
            "(J)Ljava/lang/String;",
            n_graph_sccs_json as *mut _,
        ),
        nm(
            "nGraphWeaklyConnectedComponents",
            "(J)I",
            n_graph_wccs as *mut _,
        ),
        nm("nGraphCriticalPath", "(J)I", n_graph_critical_path as *mut _),
        nm(
            "nGraphDelayWeightedDepth",
            "(J)I",
            n_graph_delay_weighted_depth as *mut _,
        ),
        nm("nGraphMaxFanIn", "(J)I", n_graph_max_fan_in as *mut _),
        nm("nGraphMaxFanOut", "(J)I", n_graph_max_fan_out as *mut _),
        nm(
            "nGraphFeaturesJson",
            "(J)Ljava/lang/String;",
            n_graph_features_json as *mut _,
        ),
        nm(
            "nGraphFingerprint",
            "(JLjava/lang/String;)Ljava/lang/String;",
            n_graph_fingerprint as *mut _,
        ),
        nm(
            "nGraphIsStructurallyEqual",
            "(JJ)Z",
            n_graph_is_structurally_equal as *mut _,
        ),
        nm(
            "nSchematicGenerateTruthTableJson",
            "(J)Ljava/lang/String;",
            n_generate_truth_table_json as *mut _,
        ),
    ];
    env.register_native_methods(&class, methods)
}

fn nm(name: &str, sig: &str, ptr: *mut c_void) -> NativeMethod {
    NativeMethod {
        name: name.into(),
        sig: sig.into(),
        fn_ptr: ptr,
    }
}

unsafe extern "system" fn n_export_graph<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    world_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let graph = as_mut::<MchprsWorld>(world_handle)
            .export_graph()
            .map_err(JvmError::Generic)?;
        Ok(to_handle(graph))
    })
}

unsafe extern "system" fn n_export_graph_structural<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    world_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let graph = as_mut::<MchprsWorld>(world_handle)
            .export_graph_structural()
            .map_err(JvmError::Generic)?;
        Ok(to_handle(graph))
    })
}

unsafe extern "system" fn n_graph_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<RedstoneGraph>(handle);
    }
}

macro_rules! graph_scalar {
    ($fn_name:ident, $ret:ty, $default:expr, $body:expr) => {
        unsafe extern "system" fn $fn_name<'l>(
            mut env: JNIEnv<'l>,
            _class: JClass<'l>,
            handle: jlong,
        ) -> $ret {
            with_jni_context(&mut env, $default, |_env| {
                let g = as_ref::<RedstoneGraph>(handle);
                #[allow(clippy::redundant_closure_call)]
                Ok(($body)(g))
            })
        }
    };
}

graph_scalar!(n_graph_node_count, jint, 0, |g: &RedstoneGraph| g.node_count() as jint);
graph_scalar!(n_graph_edge_count, jint, 0, |g: &RedstoneGraph| g.edge_count() as jint);
graph_scalar!(n_graph_has_cycles, jboolean, 0, |g: &RedstoneGraph| g.has_cycles() as jboolean);
graph_scalar!(n_graph_is_combinational, jboolean, 0, |g: &RedstoneGraph| {
    g.is_combinational() as jboolean
});
graph_scalar!(n_graph_wccs, jint, 0, |g: &RedstoneGraph| {
    g.weakly_connected_components() as jint
});
graph_scalar!(n_graph_critical_path, jint, 0, |g: &RedstoneGraph| g.critical_path() as jint);
graph_scalar!(n_graph_delay_weighted_depth, jint, 0, |g: &RedstoneGraph| {
    g.delay_weighted_depth() as jint
});
graph_scalar!(n_graph_max_fan_in, jint, 0, |g: &RedstoneGraph| g.max_fan_in() as jint);
graph_scalar!(n_graph_max_fan_out, jint, 0, |g: &RedstoneGraph| g.max_fan_out() as jint);

macro_rules! graph_json {
    ($fn_name:ident, $body:expr) => {
        unsafe extern "system" fn $fn_name<'l>(
            mut env: JNIEnv<'l>,
            _class: JClass<'l>,
            handle: jlong,
        ) -> jstring {
            with_jni_context(&mut env, std::ptr::null_mut(), |env| {
                let g = as_ref::<RedstoneGraph>(handle);
                #[allow(clippy::redundant_closure_call)]
                let json: String = ($body)(g)?;
                string_to_jstring(env, &json)
            })
        }
    };
}

graph_json!(n_graph_to_json, |g: &RedstoneGraph| g
    .to_json()
    .map_err(JvmError::Generic));
graph_json!(n_graph_nodes_json, |g: &RedstoneGraph| g
    .nodes_json()
    .map_err(JvmError::Generic));
graph_json!(n_graph_edges_json, |g: &RedstoneGraph| g
    .edges_json()
    .map_err(JvmError::Generic));
graph_json!(n_graph_features_json, |g: &RedstoneGraph| g
    .features()
    .to_json()
    .map_err(JvmError::Generic));
graph_json!(n_graph_node_kind_counts_json, |g: &RedstoneGraph| {
    serde_json::to_string(&g.node_kind_counts()).map_err(|e| JvmError::Generic(e.to_string()))
});
graph_json!(n_graph_sccs_json, |g: &RedstoneGraph| {
    serde_json::to_string(&g.strongly_connected_components())
        .map_err(|e| JvmError::Generic(e.to_string()))
});

unsafe extern "system" fn n_graph_from_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    json: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let s = jstring_to_string(env, &json)?;
        let graph = RedstoneGraph::from_json(&s).map_err(JvmError::Generic)?;
        Ok(to_handle(graph))
    })
}

unsafe extern "system" fn n_graph_fingerprint<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    preset: JString<'l>,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let preset = jstring_to_string(env, &preset)?;
        let spec = GraphFingerprintSpec::from_preset(&preset).ok_or_else(|| {
            JvmError::Generic(format!(
                "unknown fingerprint preset: {preset:?} (expected \"structural\", \"functional\", or \"exact\")"
            ))
        })?;
        let hex = as_ref::<RedstoneGraph>(handle).fingerprint(&spec).to_hex();
        string_to_jstring(env, &hex)
    })
}

unsafe extern "system" fn n_graph_is_structurally_equal<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    a: jlong,
    b: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0, |_env| {
        let ga = as_ref::<RedstoneGraph>(a);
        let gb = as_ref::<RedstoneGraph>(b);
        Ok(ga.is_structurally_equal(gb) as jboolean)
    })
}

/// Lever/lamp truth table of the schematic as JSON:
/// an array of objects mapping IO name to boolean state.
unsafe extern "system" fn n_generate_truth_table_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let schematic = as_ref::<UniversalSchematic>(schematic_handle);
        let table = generate_truth_table(schematic);
        let json = serde_json::to_string(&table).map_err(|e| JvmError::Generic(e.to_string()))?;
        string_to_jstring(env, &json)
    })
}
