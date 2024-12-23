You are a distributed systems architect for a billion-scale real-time communication platform called General Bots or gb. The system combines bot capabilities, WebRTC communication, and massive-scale messaging with the following architecture:

1. Core Domains and Models:


A. Customer Hierarchy:
- Customer (top-level organization)
  - Multiple Instances
  - Subscription Management
  - Resource Quotas
  - Regional Distribution
  - Billing & Usage Tracking

B. Instance Management:
- Per-customer instances
- Resource isolation
- Regional deployment
- Feature toggles
- Usage monitoring
- Shard management

2. Communication Infrastructure:

A. Real-time Rooms:
- WebRTC-based communication
- Track management (audio/video)
- Participant handling
- Room scaling
- Media processing
- Recording capabilities
- Video based rooms like Zoom.
- Tiktok lives - like

B. Messaging System:
- Sharded message queues
- Message persistence
- Real-time delivery
- Message routing
- Delivery status tracking
- Message search

4. Database Schema:

A. Core Tables:
```sql
CREATE TABLE customers (
    id UUID PRIMARY KEY,
    name VARCHAR(255),
    subscription_tier VARCHAR(50),
    status VARCHAR(50),
    max_instances INTEGER,
    metadata JSONB,
    created_at TIMESTAMPTZ
);

CREATE TABLE instances (
    id UUID PRIMARY KEY,
    customer_id UUID,
    name VARCHAR(255),
    status VARCHAR(50),
    shard_id INTEGER,
    region VARCHAR(50),
    config JSONB,
    created_at TIMESTAMPTZ
);

CREATE TABLE rooms (
    id UUID PRIMARY KEY,
    customer_id UUID,
    instance_id UUID,
    name VARCHAR(255),
    kind VARCHAR(50),
    status VARCHAR(50),
    config JSONB,
    created_at TIMESTAMPTZ
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    customer_id UUID,
    instance_id UUID,
    conversation_id UUID,
    sender_id UUID,
    kind VARCHAR(50),
    content TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ,
    shard_key INTEGER
);
```
Also consider every table here even if you reorganize: BOnlineSubscription
GuaribasAdmin
GuaribasAnswer
GuaribasApplications
GuaribasChannel
GuaribasConversation
GuaribasConversationMessage
GuaribasGroup
GuaribasInstance
GuaribasLog
GuaribasPackage
GuaribasQuestion
GuaribasQuestionAlternate
GuaribasSchedule
GuaribasSubject
GuaribasUser
GuaribasUserGroup


5. Scaling Architecture:

A. Storage Layer:
- PostgreSQL (relational data)
  - Sharded by customer_id
  - Partitioned tables
  - Read replicas
- TiKV (distributed KV)
  - Real-time data
  - Cache layer
  - Fast lookups
- Redis (caching)
  - Session data
  - Rate limiting
  - Temporary storage

B. Message Queue:
- Kafka clusters
  - Sharded topics
  - Message routing
  - Event streaming
- Redis Pub/Sub
  - Real-time updates
  - Presence information
  - Status changes

C. Media Handling:
- WebRTC media servers
- Track multiplexing
- Media processing
- Recording storage

6. API Structure:

A. System APIs:
```rust
pub trait SystemAPI {
    async fn call_vm(&self, pid: Uuid, text: String) -> Result<String>;
    async fn wait(&self, pid: Uuid, seconds: i32) -> Result<()>;
    async fn save_file(&self, pid: Uuid, data: Vec<u8>) -> Result<FileInfo>;
    async fn execute_sql(&self, pid: Uuid, sql: String) -> Result<QueryResult>;
}
```

B. Room APIs:
```rust
pub trait RoomAPI {
    async fn create_room(&self, config: RoomConfig) -> Result<Room>;
    async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<Connection>;
    async fn publish_track(&self, track: TrackInfo) -> Result<Track>;
    async fn subscribe_track(&self, track_id: Uuid) -> Result<Subscription>;
}
```

C. Message APIs:
```rust
pub trait MessageAPI {
    async fn send_message(&self, message: Message) -> Result<MessageId>;
    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>>;
    async fn update_status(&self, message_id: Uuid, status: Status) -> Result<()>;
}
```

7. Monitoring & Operations:

A. Metrics:
- System health
- Resource usage
- Message throughput
- Media quality
- Error rates
- API latency

B. Scaling Operations:
- Auto-scaling rules
- Shard management
- Load balancing
- Failover handling
- Data migration

C. Security:
- Authentication
- Authorization
- Rate limiting
- Data encryption
- Audit logging

Implementation Guidelines:

1. Use Rust for:
- Performance critical paths
- Memory safety
- Concurrent processing
- System reliability

2. Sharding Strategy:
- Shard by customer_id
- Instance isolation
- Regional distribution
- Data locality

3. Performance Targets:
- Billion concurrent connections
- Sub-second message delivery
- 4K video streaming
- Petabyte-scale storage

4. Reliability Requirements:
- 99.99% uptime
- No message loss
- Automatic failover
- Data redundancy

When implementing features, consider:
1. Multi-tenant isolation
2. Resource quotas
3. Security boundaries
4. Performance impact
5. Scaling implications
6. Monitoring requirements

The system should handle:
1. Billions of active users
2. Millions of concurrent rooms
3. Petabytes of message history
4. Global distribution
5. Real-time communication
6. Bot automation


API:
System Keywords:

POST /systemKeywords/callVM
POST /systemKeywords/append
POST /systemKeywords/seeCaption
POST /systemKeywords/seeText
POST /systemKeywords/sortBy
POST /systemKeywords/JSONAsGBTable
POST /systemKeywords/renderTable
POST /systemKeywords/closeHandles
POST /systemKeywords/asPDF
POST /systemKeywords/asImage
POST /systemKeywords/executeSQL
POST /systemKeywords/getFileContents
POST /systemKeywords/getRandomId
POST /systemKeywords/getStock
POST /systemKeywords/wait
POST /systemKeywords/talkTo
POST /systemKeywords/getUser
POST /systemKeywords/sendSmsTo
POST /systemKeywords/set
POST /systemKeywords/internalGetDocument
POST /systemKeywords/saveFile
POST /systemKeywords/uploadFile
POST /systemKeywords/note
POST /systemKeywords/saveToStorageBatch
POST /systemKeywords/saveToStorage
POST /systemKeywords/saveToStorageWithJSON
POST /systemKeywords/save
POST /systemKeywords/getHttp
POST /systemKeywords/isValidDate
POST /systemKeywords/isValidNumber
POST /systemKeywords/isValidHour
POST /systemKeywords/getFilter
POST /systemKeywords/find
POST /systemKeywords/getDateFromLocaleString
POST /systemKeywords/createFolder
POST /systemKeywords/shareFolder
POST /systemKeywords/internalCreateDocument
POST /systemKeywords/createDocument
POST /systemKeywords/copyFile
POST /systemKeywords/convert
POST /systemKeywords/generatePassword
POST /systemKeywords/flattenJSON
POST /systemKeywords/getCustomToken
POST /systemKeywords/getByHttp
POST /systemKeywords/putByHttp
POST /systemKeywords/postByHttp
POST /systemKeywords/numberOnly
POST /systemKeywords/createLead
POST /systemKeywords/fill
POST /systemKeywords/screenCapture
POST /systemKeywords/numberToLetters
POST /systemKeywords/getTableFromName
POST /systemKeywords/merge
POST /systemKeywords/tweet
POST /systemKeywords/rewrite
POST /systemKeywords/pay
POST /systemKeywords/autoSave
POST /systemKeywords/internalAutoSave
POST /systemKeywords/deleteFromStorage
POST /systemKeywords/deleteFile
POST /systemKeywords/getExtensionInfo
POST /systemKeywords/dirFolder
POST /systemKeywords/log
Dialog Keywords:

POST /dialogKeywords/chart
POST /dialogKeywords/getOCR
POST /dialogKeywords/getToday
POST /dialogKeywords/exit
POST /dialogKeywords/getActiveTasks
POST /dialogKeywords/createDeal
POST /dialogKeywords/findContact
POST /dialogKeywords/getContentLocaleWithCulture
POST /dialogKeywords/getCoded
POST /dialogKeywords/getWeekFromDate
POST /dialogKeywords/getDateDiff
POST /dialogKeywords/format
POST /dialogKeywords/dateAdd [...and many more dialog-related endpoints]
Web Automation:

POST /webAutomation/isSelector
POST /webAutomation/cyrb53
POST /webAutomation/closeHandles
POST /webAutomation/openPage
POST /webAutomation/getPageByHandle
POST /webAutomation/getBySelector
POST /webAutomation/getByFrame
POST /webAutomation/hover
POST /webAutomation/click [...and more web automation endpoints]

Image Processing:

POST /imageProcessing/sharpen
POST /imageProcessing/mergeImage
POST /imageProcessing/blur

Debugger Service:

There must have be a webassymbly that convert BASIC code using a compiler to webassymbly and support remotedebugging by API.

POST /debuggerService/setBreakpoint
POST /debuggerService/refactor
POST /debuggerService/resume
POST /debuggerService/stop
POST /debuggerService/step
POST /debuggerService/getContext
POST /debuggerService/start
POST /debuggerService/sendMessage

Dependencies original, migrate everything to workspace.dependencies
    "@azure/arm-appservice": "15.0.0",
    "@azure/arm-cognitiveservices": "7.5.0",
    "@azure/arm-resources": "5.2.0",
    "@azure/arm-search": "3.2.0",
    "@azure/arm-sql": "10.0.0",
    "@azure/arm-subscriptions": "5.1.0",
    "@azure/cognitiveservices-computervision": "8.2.0",
    "@azure/keyvault-keys": "4.8.0",
    "@azure/ms-rest-js": "2.7.0",
    "@azure/msal-node": "2.13.1",
    "@azure/openai": "2.0.0-beta.1",
    "@azure/search-documents": "12.1.0",
    "@azure/storage-blob": "12.24.0",
    "@google-cloud/pubsub": "4.7.0",
    "@google-cloud/translate": "8.5.0",
    "@hubspot/api-client": "11.2.0",
    "@koa/cors": "5.0.0",
    "@langchain/anthropic": "^0.3.7",
    "@langchain/community": "0.2.31",
    "@langchain/core": "^0.3.17",
    "@langchain/openai": "0.2.8",
    "@microsoft/microsoft-graph-client": "3.0.7",
    "@nlpjs/basic": "4.27.0",
    "@nosferatu500/textract": "3.1.3",
    "@push-rpc/core": "1.9.0",
    "@push-rpc/http": "1.9.0",
    "@push-rpc/openapi": "1.9.0",
    "@push-rpc/websocket": "1.9.0",
    "@semantic-release/changelog": "6.0.3",
    "@semantic-release/exec": "6.0.3",
    "@semantic-release/git": "10.0.1",
    "@sendgrid/mail": "8.1.3",
    "@sequelize/core": "7.0.0-alpha.37",
    "@types/node": "22.5.2",
    "@types/validator": "13.12.1",
    "adm-zip": "0.5.16",
    "ai2html": "^0.121.1",
    "alasql": "4.5.1",
    "any-shell-escape": "0.1.1",
    "arraybuffer-to-buffer": "0.0.7",
    "async-mutex": "0.5.0",
    "async-promises": "0.2.3",
    "async-retry": "1.3.3",
    "basic-auth": "2.0.1",
    "billboard.js": "3.13.0",
    "bluebird": "3.7.2",
    "body-parser": "1.20.2",
    "botbuilder": "4.23.0",
    "botbuilder-adapter-facebook": "1.0.12",
    "botbuilder-ai": "4.23.0",
    "botbuilder-dialogs": "4.23.0",
    "botframework-connector": "4.23.0",
    "botlib": "5.0.0",
    "c3-chart-maker": "0.2.8",
    "cd": "0.3.3",
    "chalk-animation": "2.0.3",
    "chatgpt": "5.2.5",
    "chrome-remote-interface": "0.33.2",
    "cli-progress": "3.12.0",
    "cli-spinner": "0.2.10",
    "core-js": "3.38.1",
    "cors": "2.8.5",
    "csv-database": "0.9.2",
    "data-forge": "1.10.2",
    "date-diff": "1.0.2",
    "docximager": "0.0.4",
    "docxtemplater": "3.50.0",
    "dotenv-extended": "2.9.0",
    "electron": "32.0.1",
    "exceljs": "4.4.0",
    "express": "4.19.2",
    "express-remove-route": "1.0.0",
    "facebook-nodejs-business-sdk": "^20.0.2",
    "ffmpeg-static": "5.2.0",
    "formidable": "^3.5.1",
    "get-image-colors": "4.0.1",
    "glob": "^11.0.0",
    "google-libphonenumber": "3.2.38",
    "googleapis": "143.0.0",
    "hnswlib-node": "3.0.0",
    "html-to-md": "0.8.6",
    "http-proxy": "1.18.1",
    "ibm-watson": "9.1.0",
    "icojs": "^0.19.4",
    "instagram-private-api": "1.46.1",
    "iso-639-1": "3.1.3",
    "isomorphic-fetch": "3.0.0",
    "jimp": "1.6.0",
    "js-md5": "0.8.3",
    "json-schema-to-zod": "2.4.0",
    "jsqr": "^1.4.0",
    "just-indent": "0.0.1",
    "keyv": "5.0.1",
    "koa": "2.15.3",
    "koa-body": "6.0.1",
    "koa-ratelimit": "5.1.0",
    "koa-router": "12.0.1",
    "langchain": "0.2.17",
    "language-tags": "1.0.9",
    "line-replace": "2.0.1",
    "lodash": "4.17.21",
    "luxon": "3.5.0",
    "mammoth": "1.8.0",
    "mariadb": "3.3.1",
    "mime-types": "2.1.35",
    "moment": "2.30.1",
    "ms-rest-azure": "3.0.2",
    "mysql": "^2.18.1",
    "nexmo": "2.9.1",
    "ngrok": "5.0.0-beta.2",
    "node-cron": "3.0.3",
    "node-html-parser": "6.1.13",
    "node-nlp": "4.27.0",
    "node-tesseract-ocr": "2.2.1",
    "nodemon": "^3.1.7",
    "npm": "10.8.3",
    "open": "10.1.0",
    "open-docxtemplater-image-module": "1.0.3",
    "openai": "4.57.0",
    "pdf-extraction": "1.0.2",
    "pdf-parse": "1.1.1",
    "pdf-to-png-converter": "3.3.0",
    "pdfjs-dist": "4.6.82",
    "pdfkit": "0.15.0",
    "phone": "3.1.50",
    "pizzip": "3.1.7",
    "pptxtemplater": "1.0.5",
    "pragmatismo-io-framework": "1.1.1",
    "prism-media": "1.3.5",
    "public-ip": "7.0.1",
    "punycode": "2.3.1",
    "puppeteer": "23.2.2",
    "puppeteer-extra": "3.3.6",
    "puppeteer-extra-plugin-minmax": "1.1.2",
    "puppeteer-extra-plugin-stealth": "2.11.2",
    "qr-scanner": "1.4.2",
    "qrcode": "1.5.4",
    "qrcode-reader": "^1.0.4",
    "qrcode-terminal": "0.12.0",
    "readline": "1.3.0",
    "reflect-metadata": "0.2.2",
    "rimraf": "6.0.1",
    "safe-buffer": "5.2.1",
    "scanf": "1.2.0",
    "sequelize": "6.37.3",
    "sequelize-cli": "6.6.2",
    "sequelize-typescript": "2.1.6",
    "simple-git": "3.26.0",
    "speakingurl": "14.0.1",
    "sqlite3": "5.1.7",
    "ssr-for-bots": "1.0.1-c",
    "strict-password-generator": "1.1.2",
    "svg2img": "^1.0.0-beta.2",
    "swagger-client": "3.29.2",
    "swagger-ui-dist": "5.17.14",
    "tabulator-tables": "6.2.5",
    "tedious": "18.6.1",
    "textract": "2.5.0",
    "twilio": "5.2.3",
    "twitter-api-v2": "1.17.2",
    "typeorm": "0.3.20",
    "typescript": "5.5.4",
    "url-join": "5.0.0",
    "vhost": "3.0.2",
    "vm2": "3.9.19",
    "vm2-process": "2.1.5",
    "walk-promise": "0.2.0",
    "washyourmouthoutwithsoap": "1.0.2",
    "webdav-server": "2.6.2",
    "webp-converter": "^2.3.3",
    "whatsapp-cloud-api": "0.3.1",
    "whatsapp-web.js": "1.26.1-alpha.1",
    "winston": "3.14.2",
    "ws": "8.18.0",
    "yaml": "2.5.0",
    "yarn": "1.22.22",
    "zod-to-json-schema": "3.23.2"
  },
  "devDependencies": {
    "@types/qrcode": "1.5.5",
    "@types/url-join": "4.0.3",
    "@typescript-eslint/eslint-plugin": "8.4.0",
    "@typescript-eslint/parser": "8.4.0",
    "ban-sensitive-files": "1.10.5",
    "commitizen": "4.3.0",
    "cz-conventional-changelog": "3.3.0",
    "dependency-check": "4.1.0",
    "git-issues": "1.3.1",
    "license-checker": "25.0.1",
    "prettier-standard": "16.4.1",
    "semantic-release": "24.1.0",
    "simple-commit-message": "4.1.3",
    "super-strong-password-generator": "2.0.2",
    "super-strong-password-generator-es": "2.0.2",
    "travis-deploy-once": "5.0.11",
    "tslint": "6.1.3",
    "tsx": "^4.19.1",
    "vitest": "2.0.5"

migrate them to rust compatible, 

- do not skip items, migrate everything, in way better, in your interpretation.
- use kubernetes and create environment configuration for everything and ingress to have several server nodes if eeed automatically 
- I NEED FULL CODE SOLUTION IN PROFESSIONAL TESTABLE RUST CODE: if you need split answer in several parts, but provide ENTIRE CODE. Complete working balenced aserver.  IMPORTANTE: Generate the project in a .sh shell script output with cat, of entire code base to be restored, no placeholder neither TODOS. 
- VERY IMPORNTANT: DO NOT put things like  // Add other system routes... you should WRITE ACUTAL CODE
- Need tests for every line of code written.