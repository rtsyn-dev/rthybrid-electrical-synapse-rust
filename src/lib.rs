use rtsyn_plugin::prelude::*;
use serde_json::Value;

#[derive(Debug)]
struct RthybridElectricalSynapseRust {
    post_v: f64,
    pre_v: f64,
    scale: f64,
    offset: f64,
    out_0: f64,
    g_us: f64,
}

impl Default for RthybridElectricalSynapseRust {
    fn default() -> Self {
        Self {
            post_v: 0.0,
            pre_v: 0.0,
            scale: 0.0,
            offset: 0.0,
            out_0: 0.0,
            g_us: 0.0,
        }
    }
}

impl PluginDescriptor for RthybridElectricalSynapseRust {
    fn name() -> &'static str {
        "RTHybrid Electrical Synapse"
    }

    fn kind() -> &'static str {
        "rthybrid_electrical_synapse"
    }

    fn plugin_type() -> PluginType {
        PluginType::Standard
    }

    fn inputs() -> &'static [&'static str] {
        &["Post-synaptic Voltage (V)", "Pre-synaptic Voltage (V)", "Scale (Pre to Post)", "Offset (Pre to Post)"]
    }

    fn outputs() -> &'static [&'static str] {
        &["Current (nA)"]
    }

    fn internal_variables() -> &'static [&'static str] {
        &["post_v", "pre_v", "scale", "offset", "g_us", "current"]
    }

    fn default_vars() -> Vec<(&'static str, Value)> {
        vec![("g_us", 0.0.into())]
    }

    fn behavior() -> PluginBehavior {
        PluginBehavior {
            supports_start_stop: true,
            supports_restart: true,
            supports_apply: false,
            extendable_inputs: ExtendableInputs::None,
            loads_started: false,
            external_window: false,
            starts_expanded: true,
            start_requires_connected_inputs: Vec::new(),
            start_requires_connected_outputs: Vec::new(),
        }
    }
}

impl PluginRuntime for RthybridElectricalSynapseRust {
    fn set_config_value(&mut self, key: &str, value: &Value) {
        let Some(v) = value.as_f64() else {
            return;
        };
        match key {
            "g_us" | "g_gap" | "g (uS)" | "g (microS)" | "g" => self.g_us = v,
            "g (nS)" => self.g_us = v / 1000.0,
            _ => {}
        }
    }

    fn set_input_value(&mut self, key: &str, v: f64) {
        match key {
            "Post-synaptic Voltage (V)" => self.post_v = if v.is_finite() { v } else { 0.0 },
            "Pre-synaptic Voltage (V)" => self.pre_v = if v.is_finite() { v } else { 0.0 },
            "Scale (Pre to Post)" => self.scale = if v.is_finite() { v } else { 0.0 },
            "Offset (Pre to Post)" => self.offset = if v.is_finite() { v } else { 0.0 },
            _ => {}
        }
    }

    fn process_tick(&mut self, _tick: u64, period_seconds: f64) {
        let _ = period_seconds;
        if !self.g_us.is_finite()
            || !self.post_v.is_finite()
            || !self.pre_v.is_finite()
            || !self.scale.is_finite()
            || !self.offset.is_finite()
        {
            self.out_0 = 0.0;
            return;
        }
        let mut scale = self.scale;
        let mut offset_mv = self.offset * 1000.0;
        if scale.abs() < 1e-15 {
            scale = 1.0;
            offset_mv = 0.0;
        }
        scale = scale.clamp(-1e6, 1e6);
        offset_mv = offset_mv.clamp(-1e6, 1e6);
        let v_post_mv = self.post_v * 1000.0;
        let v_pre_mv = self.pre_v * 1000.0 * scale + offset_mv;
        self.out_0 = (self.g_us * (v_post_mv - v_pre_mv)).clamp(-1e6, 1e6);
        if !self.out_0.is_finite() {
            self.out_0 = 0.0;
        }
    }

    fn get_output_value(&self, key: &str) -> f64 {
        match key {
            "Current (nA)" => self.out_0,
            _ => 0.0,
        }
    }

    fn get_internal_value(&self, key: &str) -> Option<f64> {
        match key {
            "post_v" => Some(self.post_v),
            "pre_v" => Some(self.pre_v),
            "scale" => Some(self.scale),
            "offset" => Some(self.offset),
            "g_us" => Some(self.g_us),
            "current" => Some(self.out_0),
            _ => None,
        }
    }
}

rtsyn_plugin::export_plugin!(RthybridElectricalSynapseRust);
