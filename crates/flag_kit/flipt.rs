use std::collections::HashMap;

use anyhow::Result;
use flipt::{
    Config, ConfigBuilder, NoneAuthentication,
    api::FliptClient,
    error::UpstreamError,
    evaluation::models::{EvaluationRequest, VariantEvaluationResponse},
};
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationResult, StructValue, Value,
    async_trait,
    provider::{FeatureProvider, ProviderMetadata, ResolutionDetails},
};
use reqwest::header::{HeaderMap, HeaderValue};

const DEFAULT_ENTITY_ID: &str = "";
const METADATA: &str = "flipt";

pub struct FliptProvider {
    client: FliptClient,
    metadata: ProviderMetadata,
    namespace: String,
}

impl FliptProvider {
    pub fn try_new(environment: impl AsRef<str>, namespace: impl AsRef<str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Flipt-Environment",
            HeaderValue::from_str(environment.as_ref())?,
        );
        let config: Config<_> = ConfigBuilder::default()
            .with_endpoint("http://localhost:8081".parse()?)
            .with_auth_strategy(NoneAuthentication::new())
            .with_timeout(std::time::Duration::from_secs(10))
            .with_headers(headers)
            .build();
        let client = FliptClient::new(config)?;
        Ok(Self {
            metadata: ProviderMetadata::new(METADATA),
            namespace: namespace.as_ref().to_string(),
            client,
        })
    }
}

#[async_trait]
impl FeatureProvider for FliptProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        self.client
            .evaluation
            .boolean(&EvaluationRequest {
                namespace_key: self.namespace.clone(),
                flag_key: flag_key.into(),
                entity_id: evaluation_context
                    .targeting_key
                    .clone()
                    .unwrap_or(DEFAULT_ENTITY_ID.to_owned()),
                context: translate_context(evaluation_context),
                reference: None,
            })
            .await
            .map_err(translate_error)
            .map(|v| ResolutionDetails::new(v.enabled))
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        let res = variant_helper(self, flag_key, evaluation_context).await?;
        // parse a variant key as i64
        res.variant_key
            .parse::<i64>()
            .map_err(|e| EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a number in range of i64, but found `{}` ({:?})",
                    res.variant_attachment, e
                )),
            })
            .map(ResolutionDetails::new)
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        let res = variant_helper(self, flag_key, evaluation_context).await?;
        // parse a variant key as f64
        res.variant_key
            .parse::<f64>()
            .map_err(|e| EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a number in range of f64, but found `{}` ({:?})",
                    res.variant_attachment, e
                )),
            })
            .map(ResolutionDetails::new)
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        let res = variant_helper(self, flag_key, evaluation_context).await?;
        Ok(ResolutionDetails::new(res.variant_key))
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        let res = variant_helper(self, flag_key, evaluation_context).await?;
        // parse a variant attachment as a struct value
        let v = parse_json(&res.variant_attachment)?;
        if let Value::Struct(sv) = v {
            Ok(ResolutionDetails::new(sv))
        } else {
            Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error".to_owned()),
                message: Some(format!(
                    "Expected a struct value, but found `{}`",
                    res.variant_attachment
                )),
            })
        }
    }
}

fn translate_error(e: UpstreamError) -> EvaluationError {
    EvaluationError {
        code: EvaluationErrorCode::General(format!(
            "Flipt error: {}, message: \"{}\"",
            e.code, e.message
        )),
        message: Some(format!("{}", e)),
    }
}

fn translate_context(ctx: &EvaluationContext) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, v) in ctx.custom_fields.iter() {
        if let Some(v) = v.as_str() {
            map.insert(k.clone(), v.to_owned());
        };
    }
    map
}

async fn variant_helper(
    provider: &FliptProvider,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<VariantEvaluationResponse, EvaluationError> {
    provider
        .client
        .evaluation
        .variant(&EvaluationRequest {
            namespace_key: provider.namespace.clone(),
            flag_key: flag_key.into(),
            entity_id: ctx
                .targeting_key
                .clone()
                .unwrap_or(DEFAULT_ENTITY_ID.to_owned()),
            context: translate_context(ctx),
            reference: None,
        })
        .await
        .map_err(translate_error)
}

fn parse_json(json: &str) -> Result<Value, EvaluationError> {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
                message: Some(format!("Failed to parse JSON: {}", e)),
            });
        }
    };
    serde_to_openfeature_value(v)
}

fn serde_to_openfeature_value(v: serde_json::Value) -> Result<Value, EvaluationError> {
    match v {
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            let opt = if n.is_i64() {
                n.as_i64().map(Value::Int)
            } else if n.is_f64() {
                n.as_f64().map(Value::Float)
            } else {
                None
            };
            opt.map(Ok).unwrap_or(Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
                message: Some(format!(
                    "Expected a number of type i64 or f64, but found `{}`",
                    n
                )),
            }))
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Null => Err(EvaluationError {
            code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
            message: Some(format!("Unsupported JSON value: {}", v)),
        }),
        serde_json::Value::Array(a) => {
            let mut arr = Vec::new();
            for v in a {
                arr.push(serde_to_openfeature_value(v)?);
            }
            Ok(Value::Array(arr))
        }
        serde_json::Value::Object(o) => {
            let mut map = HashMap::new();
            for (k, v) in o {
                map.insert(k, serde_to_openfeature_value(v)?);
            }
            Ok(Value::Struct(StructValue { fields: map }))
        }
    }
}
