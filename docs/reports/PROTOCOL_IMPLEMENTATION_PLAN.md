# Cap'n Web Protocol Implementation Plan

## Executive Summary

After thorough study of the Cap'n Web protocol specification, this document outlines the comprehensive plan to correct our Rust implementation to match the actual protocol exactly.

## Key Protocol Insights

### 1. Message Structure
Cap'n Web messages are **arrays** with the type as the first element:
```json
["push", expression]
["pull", importId]
["resolve", exportId, expression]
["reject", exportId, expression]
["release", importId, refcount]
["abort", expression]
```

### 2. Expression System
Expressions are JSON trees with special evaluation rules:
- Literal JSON types (except arrays) are passed through
- Arrays have special meanings based on their first element
- Supports imports, pipelines, remapping, exports, and promises

### 3. Import/Export Tables
- **Imports**: Positive IDs (1, 2, 3...)
- **Exports**: Negative IDs (-1, -2, -3...)
- **Main interface**: ID 0
- IDs are never reused

### 4. Promise Pipelining
Built directly into the protocol via expression evaluation and the "pipeline" expression type.

## Implementation Strategy

### Phase 1: Core Protocol Types (Week 1)

#### 1.1 Message Type System
```rust
// capnweb-core/src/protocol/message.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Push(PushMessage),
    Pull(PullMessage),
    Resolve(ResolveMessage),
    Reject(RejectMessage),
    Release(ReleaseMessage),
    Abort(AbortMessage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushMessage(pub String, pub Expression);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PullMessage(pub String, pub ImportId);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolveMessage(pub String, pub ExportId, pub Expression);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectMessage(pub String, pub ExportId, pub Expression);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseMessage(pub String, pub ImportId, pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbortMessage(pub String, pub Expression);
```

#### 1.2 Expression System
```rust
// capnweb-core/src/protocol/expression.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Expression {
    // Literal JSON values
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Object(Map<String, Expression>),

    // Special array-based expressions
    Array(Vec<Expression>),

    // Will be parsed from arrays based on first element
    EscapedArray(Box<Vec<Expression>>),
    Date(f64),
    Error(ErrorExpression),
    Import(ImportExpression),
    Pipeline(PipelineExpression),
    Remap(RemapExpression),
    Export(ExportExpression),
    Promise(PromiseExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub call_arguments: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub call_arguments: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemapExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub captures: Vec<CaptureRef>,
    pub instructions: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportExpression {
    pub export_id: ExportId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromiseExpression {
    pub export_id: ExportId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorExpression {
    pub error_type: String,
    pub message: String,
    pub stack: Option<String>,
}
```

#### 1.3 Import/Export ID Management
```rust
// capnweb-core/src/protocol/ids.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportId(pub i64);

impl ImportId {
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    pub fn is_local(&self) -> bool {
        self.0 > 0
    }

    pub fn is_remote(&self) -> bool {
        self.0 < 0
    }
}

impl ExportId {
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    pub fn is_local(&self) -> bool {
        self.0 < 0
    }

    pub fn is_remote(&self) -> bool {
        self.0 > 0
    }
}
```

### Phase 2: Expression Evaluation Engine (Week 1-2)

#### 2.1 Expression Parser
```rust
// capnweb-core/src/protocol/parser.rs
pub struct ExpressionParser;

impl ExpressionParser {
    pub fn parse_value(value: &serde_json::Value) -> Result<Expression, ParseError> {
        match value {
            serde_json::Value::Array(arr) if !arr.is_empty() => {
                match &arr[0] {
                    serde_json::Value::String(type_code) => {
                        Self::parse_typed_array(type_code, arr)
                    }
                    serde_json::Value::Array(_) => {
                        // Escaped array: [[...]]
                        Ok(Expression::EscapedArray(Box::new(
                            Self::parse_array_elements(&arr[0])?
                        )))
                    }
                    _ => Ok(Expression::Array(Self::parse_array_elements(arr)?))
                }
            }
            serde_json::Value::Array(arr) => {
                Ok(Expression::Array(Vec::new()))
            }
            serde_json::Value::Null => Ok(Expression::Null),
            serde_json::Value::Bool(b) => Ok(Expression::Bool(*b)),
            serde_json::Value::Number(n) => Ok(Expression::Number(n.clone())),
            serde_json::Value::String(s) => Ok(Expression::String(s.clone())),
            serde_json::Value::Object(obj) => {
                let mut map = Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Self::parse_value(v)?);
                }
                Ok(Expression::Object(map))
            }
        }
    }

    fn parse_typed_array(type_code: &str, arr: &[serde_json::Value]) -> Result<Expression, ParseError> {
        match type_code {
            "date" => {
                // ["date", milliseconds]
                if arr.len() != 2 {
                    return Err(ParseError::InvalidDateExpression);
                }
                let millis = arr[1].as_f64()
                    .ok_or(ParseError::InvalidDateExpression)?;
                Ok(Expression::Date(millis))
            }
            "error" => {
                // ["error", type, message, stack?]
                if arr.len() < 3 || arr.len() > 4 {
                    return Err(ParseError::InvalidErrorExpression);
                }
                Ok(Expression::Error(ErrorExpression {
                    error_type: arr[1].as_str()
                        .ok_or(ParseError::InvalidErrorExpression)?.to_string(),
                    message: arr[2].as_str()
                        .ok_or(ParseError::InvalidErrorExpression)?.to_string(),
                    stack: arr.get(3).and_then(|v| v.as_str()).map(String::from),
                }))
            }
            "import" => Self::parse_import_expression(arr, false),
            "pipeline" => Self::parse_import_expression(arr, true),
            "remap" => Self::parse_remap_expression(arr),
            "export" => Self::parse_export_expression(arr),
            "promise" => Self::parse_promise_expression(arr),
            _ => {
                // Unknown type code, treat as regular array
                Ok(Expression::Array(Self::parse_array_elements(arr)?))
            }
        }
    }
}
```

#### 2.2 Expression Evaluator
```rust
// capnweb-core/src/protocol/evaluator.rs
pub struct ExpressionEvaluator {
    imports: ImportTable,
    exports: ExportTable,
    resolver: Box<dyn CapabilityResolver>,
}

impl ExpressionEvaluator {
    pub async fn evaluate(&mut self, expr: Expression) -> Result<Value, EvalError> {
        match expr {
            Expression::Null => Ok(Value::Null),
            Expression::Bool(b) => Ok(Value::Bool(b)),
            Expression::Number(n) => Ok(Value::Number(n)),
            Expression::String(s) => Ok(Value::String(s)),
            Expression::Object(map) => {
                let mut result = Map::new();
                for (k, v) in map {
                    result.insert(k, self.evaluate(v).await?);
                }
                Ok(Value::Object(result))
            }
            Expression::Array(elements) => {
                let mut result = Vec::new();
                for elem in elements {
                    result.push(self.evaluate(elem).await?);
                }
                Ok(Value::Array(result))
            }
            Expression::EscapedArray(elements) => {
                // Return as literal array
                let mut result = Vec::new();
                for elem in *elements {
                    result.push(self.evaluate(elem).await?);
                }
                Ok(Value::Array(result))
            }
            Expression::Date(millis) => {
                Ok(Value::Date(millis))
            }
            Expression::Error(err) => {
                Ok(Value::Error(err))
            }
            Expression::Import(import) => {
                self.evaluate_import(import, false).await
            }
            Expression::Pipeline(pipeline) => {
                self.evaluate_import(pipeline.into(), true).await
            }
            Expression::Remap(remap) => {
                self.evaluate_remap(remap).await
            }
            Expression::Export(export) => {
                self.evaluate_export(export).await
            }
            Expression::Promise(promise) => {
                self.evaluate_promise(promise).await
            }
        }
    }

    async fn evaluate_import(
        &mut self,
        import: ImportExpression,
        as_promise: bool
    ) -> Result<Value, EvalError> {
        let target = self.imports.get(import.import_id)?;

        let mut result = if as_promise {
            target.as_promise()
        } else {
            target.as_stub()
        };

        // Apply property path
        if let Some(path) = import.property_path {
            for key in path {
                result = result.get_property(key)?;
            }
        }

        // Apply call if requested
        if let Some(args_expr) = import.call_arguments {
            let args = self.evaluate(*args_expr).await?;
            result = result.call(args).await?;
        }

        Ok(result)
    }
}
```

### Phase 3: Import/Export Table Management (Week 2)

#### 3.1 Import Table
```rust
// capnweb-core/src/protocol/tables.rs
pub struct ImportTable {
    next_positive_id: AtomicI64,
    next_negative_id: AtomicI64,
    entries: DashMap<ImportId, ImportEntry>,
}

impl ImportTable {
    pub fn new() -> Self {
        Self {
            next_positive_id: AtomicI64::new(1),
            next_negative_id: AtomicI64::new(-1),
            entries: DashMap::new(),
        }
    }

    pub fn allocate_local(&self) -> ImportId {
        ImportId(self.next_positive_id.fetch_add(1, Ordering::SeqCst))
    }

    pub fn allocate_remote(&self) -> ImportId {
        ImportId(self.next_negative_id.fetch_sub(1, Ordering::SeqCst))
    }

    pub fn insert(&self, id: ImportId, entry: ImportEntry) {
        self.entries.insert(id, entry);
    }

    pub fn get(&self, id: ImportId) -> Option<ImportEntry> {
        self.entries.get(&id).map(|e| e.clone())
    }

    pub fn release(&self, id: ImportId, refcount: u32) -> bool {
        if let Some(mut entry) = self.entries.get_mut(&id) {
            entry.refcount = entry.refcount.saturating_sub(refcount);
            if entry.refcount == 0 {
                self.entries.remove(&id);
                return true;
            }
        }
        false
    }
}

pub struct ImportEntry {
    pub value: ImportValue,
    pub refcount: u32,
}

pub enum ImportValue {
    Stub(Arc<dyn RpcTarget>),
    Promise(Promise<Value>),
    Resolved(Value),
}
```

#### 3.2 Export Table
```rust
pub struct ExportTable {
    next_negative_id: AtomicI64,
    next_positive_id: AtomicI64,
    entries: DashMap<ExportId, ExportEntry>,
}

impl ExportTable {
    pub fn new() -> Self {
        Self {
            next_negative_id: AtomicI64::new(-1),
            next_positive_id: AtomicI64::new(1),
            entries: DashMap::new(),
        }
    }

    pub fn allocate_local(&self) -> ExportId {
        ExportId(self.next_negative_id.fetch_sub(1, Ordering::SeqCst))
    }

    pub fn allocate_remote(&self) -> ExportId {
        ExportId(self.next_positive_id.fetch_add(1, Ordering::SeqCst))
    }

    pub fn insert(&self, id: ExportId, entry: ExportEntry) {
        self.entries.insert(id, entry);
    }

    pub fn get(&self, id: ExportId) -> Option<ExportEntry> {
        self.entries.get(&id).map(|e| e.clone())
    }

    pub fn resolve(&self, id: ExportId, value: Value) -> Result<(), ResolveError> {
        if let Some(mut entry) = self.entries.get_mut(&id) {
            entry.resolve(value)?;
            Ok(())
        } else {
            Err(ResolveError::UnknownExport(id))
        }
    }

    pub fn reject(&self, id: ExportId, error: Value) -> Result<(), ResolveError> {
        if let Some(mut entry) = self.entries.get_mut(&id) {
            entry.reject(error)?;
            Ok(())
        } else {
            Err(ResolveError::UnknownExport(id))
        }
    }
}

pub struct ExportEntry {
    pub value: ExportValue,
    pub export_count: u32,
}

pub enum ExportValue {
    Stub(Arc<dyn RpcTarget>),
    Promise(oneshot::Sender<Result<Value, Value>>),
    Resolved(Value),
    Rejected(Value),
}
```

### Phase 4: Session Management (Week 2-3)

#### 4.1 RPC Session
```rust
// capnweb-core/src/protocol/session.rs
pub struct RpcSession {
    imports: Arc<ImportTable>,
    exports: Arc<ExportTable>,
    evaluator: Arc<Mutex<ExpressionEvaluator>>,
    transport: Box<dyn Transport>,
    pending_pulls: DashMap<ImportId, oneshot::Sender<Value>>,
}

impl RpcSession {
    pub async fn handle_message(&self, msg: Message) -> Result<(), SessionError> {
        match msg {
            Message::Push(push) => {
                let import_id = self.imports.allocate_local();
                let expr = push.1;

                // Evaluate the expression
                let result = self.evaluator.lock().await.evaluate(expr).await?;

                // Store in import table as promise
                self.imports.insert(import_id, ImportEntry {
                    value: ImportValue::Promise(Promise::new(result)),
                    refcount: 1,
                });
            }

            Message::Pull(pull) => {
                let import_id = pull.1;

                // Check if we have this import
                if let Some(entry) = self.imports.get(import_id) {
                    match entry.value {
                        ImportValue::Promise(promise) => {
                            // Wait for promise resolution
                            let value = promise.await?;

                            // Send resolve message
                            let export_id = ExportId(-import_id.0);
                            self.send_resolve(export_id, value).await?;
                        }
                        ImportValue::Resolved(value) => {
                            // Already resolved, send immediately
                            let export_id = ExportId(-import_id.0);
                            self.send_resolve(export_id, value).await?;
                        }
                        _ => return Err(SessionError::InvalidPull),
                    }
                }
            }

            Message::Resolve(resolve) => {
                let export_id = resolve.1;
                let expr = resolve.2;

                // Evaluate the expression
                let value = self.evaluator.lock().await.evaluate(expr).await?;

                // Resolve the export
                self.exports.resolve(export_id, value)?;

                // Notify any waiting pulls
                if let ImportId(id) = ImportId(-export_id.0) {
                    if let Some((_, sender)) = self.pending_pulls.remove(&ImportId(id)) {
                        let _ = sender.send(value);
                    }
                }
            }

            Message::Reject(reject) => {
                let export_id = reject.1;
                let expr = reject.2;

                // Evaluate the error expression
                let error = self.evaluator.lock().await.evaluate(expr).await?;

                // Reject the export
                self.exports.reject(export_id, error)?;
            }

            Message::Release(release) => {
                let import_id = release.1;
                let refcount = release.2;

                // Release the import
                if self.imports.release(import_id, refcount) {
                    // Import was fully released, clean up
                    self.cleanup_import(import_id).await?;
                }
            }

            Message::Abort(abort) => {
                let expr = abort.1;

                // Evaluate the error expression
                let error = self.evaluator.lock().await.evaluate(expr).await?;

                // Terminate the session
                self.abort_session(error).await?;
            }
        }

        Ok(())
    }

    pub async fn push(&self, expr: Expression) -> Result<ImportId, SessionError> {
        let import_id = self.imports.allocate_local();

        // Send push message
        let msg = Message::Push(PushMessage("push".to_string(), expr));
        self.transport.send(msg).await?;

        Ok(import_id)
    }

    pub async fn pull(&self, import_id: ImportId) -> Result<Value, SessionError> {
        // Send pull message
        let msg = Message::Pull(PullMessage("pull".to_string(), import_id));
        self.transport.send(msg).await?;

        // Wait for resolution
        let (tx, rx) = oneshot::channel();
        self.pending_pulls.insert(import_id, tx);

        Ok(rx.await?)
    }
}
```

### Phase 5: Transport Layer Updates (Week 3)

#### 5.1 Message Serialization
```rust
// capnweb-transport/src/serialization.rs
pub struct CapnWebCodec;

impl CapnWebCodec {
    pub fn encode_message(msg: &Message) -> Result<Vec<u8>, CodecError> {
        // Messages are encoded as JSON arrays
        let json_value = match msg {
            Message::Push(push) => {
                json!(["push", Self::expression_to_json(&push.1)?])
            }
            Message::Pull(pull) => {
                json!(["pull", pull.1.0])
            }
            Message::Resolve(resolve) => {
                json!(["resolve", resolve.1.0, Self::expression_to_json(&resolve.2)?])
            }
            Message::Reject(reject) => {
                json!(["reject", reject.1.0, Self::expression_to_json(&reject.2)?])
            }
            Message::Release(release) => {
                json!(["release", release.1.0, release.2])
            }
            Message::Abort(abort) => {
                json!(["abort", Self::expression_to_json(&abort.1)?])
            }
        };

        Ok(serde_json::to_vec(&json_value)?)
    }

    pub fn decode_message(data: &[u8]) -> Result<Message, CodecError> {
        let json_value: serde_json::Value = serde_json::from_slice(data)?;

        // Parse JSON array into message
        if let serde_json::Value::Array(arr) = json_value {
            if arr.is_empty() {
                return Err(CodecError::EmptyMessage);
            }

            let msg_type = arr[0].as_str()
                .ok_or(CodecError::InvalidMessageType)?;

            match msg_type {
                "push" => {
                    if arr.len() != 2 {
                        return Err(CodecError::InvalidPushMessage);
                    }
                    let expr = ExpressionParser::parse_value(&arr[1])?;
                    Ok(Message::Push(PushMessage("push".to_string(), expr)))
                }
                "pull" => {
                    if arr.len() != 2 {
                        return Err(CodecError::InvalidPullMessage);
                    }
                    let import_id = arr[1].as_i64()
                        .ok_or(CodecError::InvalidImportId)?;
                    Ok(Message::Pull(PullMessage("pull".to_string(), ImportId(import_id))))
                }
                "resolve" => {
                    if arr.len() != 3 {
                        return Err(CodecError::InvalidResolveMessage);
                    }
                    let export_id = arr[1].as_i64()
                        .ok_or(CodecError::InvalidExportId)?;
                    let expr = ExpressionParser::parse_value(&arr[2])?;
                    Ok(Message::Resolve(ResolveMessage(
                        "resolve".to_string(),
                        ExportId(export_id),
                        expr
                    )))
                }
                "reject" => {
                    if arr.len() != 3 {
                        return Err(CodecError::InvalidRejectMessage);
                    }
                    let export_id = arr[1].as_i64()
                        .ok_or(CodecError::InvalidExportId)?;
                    let expr = ExpressionParser::parse_value(&arr[2])?;
                    Ok(Message::Reject(RejectMessage(
                        "reject".to_string(),
                        ExportId(export_id),
                        expr
                    )))
                }
                "release" => {
                    if arr.len() != 3 {
                        return Err(CodecError::InvalidReleaseMessage);
                    }
                    let import_id = arr[1].as_i64()
                        .ok_or(CodecError::InvalidImportId)?;
                    let refcount = arr[2].as_u64()
                        .ok_or(CodecError::InvalidRefcount)? as u32;
                    Ok(Message::Release(ReleaseMessage(
                        "release".to_string(),
                        ImportId(import_id),
                        refcount
                    )))
                }
                "abort" => {
                    if arr.len() != 2 {
                        return Err(CodecError::InvalidAbortMessage);
                    }
                    let expr = ExpressionParser::parse_value(&arr[1])?;
                    Ok(Message::Abort(AbortMessage("abort".to_string(), expr)))
                }
                _ => Err(CodecError::UnknownMessageType(msg_type.to_string()))
            }
        } else {
            Err(CodecError::NotAnArray)
        }
    }
}
```

### Phase 6: Promise Pipelining Implementation (Week 3-4)

#### 6.1 Pipeline Support
```rust
// capnweb-core/src/protocol/pipeline.rs
pub struct PipelineManager {
    promises: DashMap<ImportId, PromiseState>,
}

pub enum PromiseState {
    Pending(Vec<PipelineOperation>),
    Resolved(Value),
    Rejected(Value),
}

pub struct PipelineOperation {
    pub property_path: Option<Vec<PropertyKey>>,
    pub call_arguments: Option<Box<Expression>>,
    pub result_id: ImportId,
}

impl PipelineManager {
    pub fn register_pipeline(
        &self,
        base_id: ImportId,
        operation: PipelineOperation
    ) -> ImportId {
        let result_id = operation.result_id;

        self.promises.alter(&base_id, |_, state| {
            match state {
                Some(PromiseState::Pending(mut ops)) => {
                    ops.push(operation);
                    PromiseState::Pending(ops)
                }
                Some(PromiseState::Resolved(value)) => {
                    // Promise already resolved, execute immediately
                    self.execute_pipeline_on_value(value, operation);
                    PromiseState::Resolved(value)
                }
                Some(PromiseState::Rejected(error)) => {
                    // Promise rejected, propagate error
                    self.propagate_rejection(result_id, error);
                    PromiseState::Rejected(error)
                }
                None => {
                    // New promise, start tracking
                    PromiseState::Pending(vec![operation])
                }
            }
        });

        result_id
    }

    pub fn resolve_promise(&self, id: ImportId, value: Value) {
        if let Some(mut entry) = self.promises.get_mut(&id) {
            let operations = match &*entry {
                PromiseState::Pending(ops) => ops.clone(),
                _ => return, // Already resolved
            };

            *entry = PromiseState::Resolved(value.clone());

            // Execute all pending pipeline operations
            for op in operations {
                self.execute_pipeline_on_value(value.clone(), op);
            }
        }
    }
}
```

### Phase 7: Testing and Validation (Week 4)

#### 7.1 Protocol Compliance Tests
```rust
// tests/protocol_compliance.rs
#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        // Test all message types serialize correctly
        let push = Message::Push(PushMessage(
            "push".to_string(),
            Expression::String("test".to_string())
        ));

        let encoded = CapnWebCodec::encode_message(&push).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(json, json!(["push", "test"]));
    }

    #[test]
    fn test_expression_evaluation() {
        // Test expression parsing and evaluation
        let json = json!(["import", 1, ["property"], [1, 2, 3]]);
        let expr = ExpressionParser::parse_value(&json).unwrap();

        match expr {
            Expression::Import(import) => {
                assert_eq!(import.import_id, ImportId(1));
                assert_eq!(import.property_path, Some(vec![PropertyKey::String("property".to_string())]));
                assert!(import.call_arguments.is_some());
            }
            _ => panic!("Expected Import expression"),
        }
    }

    #[test]
    fn test_import_export_tables() {
        // Test ID allocation and management
        let imports = ImportTable::new();

        let id1 = imports.allocate_local();
        assert_eq!(id1, ImportId(1));

        let id2 = imports.allocate_local();
        assert_eq!(id2, ImportId(2));

        let id3 = imports.allocate_remote();
        assert_eq!(id3, ImportId(-1));
    }

    #[tokio::test]
    async fn test_promise_pipelining() {
        // Test promise pipelining functionality
        let pipeline = PipelineManager::new();

        let base_id = ImportId(1);
        let op = PipelineOperation {
            property_path: Some(vec![PropertyKey::String("method".to_string())]),
            call_arguments: None,
            result_id: ImportId(2),
        };

        let result_id = pipeline.register_pipeline(base_id, op);
        assert_eq!(result_id, ImportId(2));

        // Resolve the base promise
        pipeline.resolve_promise(base_id, Value::String("resolved".to_string()));

        // Check that pipeline was executed
        // (Additional verification needed)
    }
}
```

#### 7.2 TypeScript Client Integration Tests
```rust
// tests/typescript_integration.rs
#[cfg(test)]
mod typescript_tests {
    use super::*;

    #[tokio::test]
    async fn test_with_official_client() {
        // Start server with correct protocol implementation
        let server = CapnWebServer::new(ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
        });

        // Register main interface (ID 0)
        server.register_main(Arc::new(Calculator));

        let handle = tokio::spawn(async move {
            server.run().await
        });

        // Run TypeScript client tests
        let output = Command::new("npm")
            .arg("test")
            .current_dir("typescript-interop")
            .output()
            .expect("Failed to run TypeScript tests");

        assert!(output.status.success(), "TypeScript tests failed");

        handle.abort();
    }
}
```

## Task List

### Week 1: Core Protocol Implementation
- [ ] Implement new message types (Push, Pull, Resolve, Reject, Release, Abort)
- [ ] Create expression system with proper serialization
- [ ] Implement expression parser for all expression types
- [ ] Add import/export ID management system
- [ ] Create basic expression evaluator

### Week 2: Session and Table Management
- [ ] Implement import table with refcounting
- [ ] Implement export table with promise tracking
- [ ] Create RPC session manager
- [ ] Add bidirectional message handling
- [ ] Implement proper ID allocation strategy

### Week 3: Transport and Pipelining
- [ ] Update transport layer for new message format
- [ ] Implement CapnWebCodec for serialization
- [ ] Add promise pipelining support
- [ ] Create pipeline manager
- [ ] Implement expression evaluation with pipelining

### Week 4: Testing and Validation
- [ ] Write protocol compliance tests
- [ ] Test all message types
- [ ] Validate expression evaluation
- [ ] Test promise pipelining
- [ ] Run TypeScript client integration tests

## Success Criteria

1. **Protocol Compliance**
   - All messages match Cap'n Web specification exactly
   - Expression evaluation works correctly
   - Import/export tables function as specified

2. **TypeScript Client Compatibility**
   - Official client can connect and communicate
   - All RPC patterns work correctly
   - Promise pipelining demonstrates single round-trip optimization

3. **Feature Completeness**
   - Bidirectional communication works
   - Promise pipelining functional
   - Remap operations supported
   - Error handling matches specification

## Next Steps

1. Begin implementation of core protocol types
2. Replace existing message system with Cap'n Web format
3. Update transport layer to handle new message format
4. Validate each component against specification
5. Test with official TypeScript client

The implementation will be done incrementally, with each phase building on the previous one. Regular testing against the specification will ensure compliance throughout the development process.