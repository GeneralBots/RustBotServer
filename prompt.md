You are a distributed systems architect for a billion-scale real-time communication platform called General Bots or gb. The system combines bot capabilities, WebRTC communication, and massive-scale messaging with the following architecture:

1. Core Domains and Models:

CREATE TABLE "GBOnlineSubscription" (
	"Id" serial4 NOT NULL,
	"instanceId" int4 NULL,
	"externalSubscriptionId" varchar(255) NULL,
	"saasSubscriptionStatus" varchar(255) NULL,
	"isFreeTrial" bool NULL,
	"planId" varchar(255) NULL,
	quantity int4 NULL,
	"lastCCFourDigits" int4 NULL,
	status varchar(255) NULL,
	CONSTRAINT "GBOnlineSubscription_pkey" PRIMARY KEY ("Id")
);

CREATE TABLE "GuaribasAdmin" (
	id serial4 NOT NULL,
	"instanceId" int4 NULL,
	"key" varchar(255) NULL,
	value varchar(4000) NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	CONSTRAINT "GuaribasAdmin_pkey" PRIMARY KEY (id)
);


CREATE TABLE "GuaribasChannel" (
	"channelId" serial4 NOT NULL,
	title varchar(255) NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	CONSTRAINT "GuaribasChannel_pkey" PRIMARY KEY ("channelId")
);


-- public."GuaribasInstance" definition

-- Drop table

-- DROP TABLE "GuaribasInstance";

CREATE TABLE "GuaribasInstance" (
	"instanceId" serial4 NOT NULL,
	"botEndpoint" varchar(255) NULL,
	"whoAmIVideo" varchar(255) NULL,
	"botId" varchar(255) NULL,
	title varchar(255) NULL,
	"activationCode" varchar(16) NULL,
	description varchar(255) NULL,
	state varchar(16) NULL,
	"botKey" varchar(64) NULL,
	"enabledAdmin" varchar(255) NULL,
	"engineName" varchar(255) NULL,
	"marketplaceId" varchar(255) NULL,
	"textAnalyticsKey" varchar(255) NULL,
	"textAnalyticsEndpoint" varchar(255) NULL,
	"translatorKey" varchar(64) NULL,
	"translatorEndpoint" varchar(128) NULL,
	"marketplacePassword" varchar(255) NULL,
	"webchatKey" varchar(255) NULL,
	"authenticatorTenant" varchar(255) NULL,
	"authenticatorAuthorityHostUrl" varchar(255) NULL,
	"cloudSubscriptionId" varchar(255) NULL,
	"cloudUsername" varchar(255) NULL,
	"cloudPassword" varchar(255) NULL,
	"cloudLocation" varchar(255) NULL,
	"googleBotKey" varchar(255) NULL,
	"googleChatApiKey" varchar(255) NULL,
	"googleChatSubscriptionName" varchar(255) NULL,
	"googleClientEmail" varchar(255) NULL,
	"googlePrivateKey" varchar(4000) NULL,
	"googleProjectId" varchar(255) NULL,
	"facebookWorkplaceVerifyToken" varchar(255) NULL,
	"facebookWorkplaceAppSecret" varchar(255) NULL,
	"facebookWorkplaceAccessToken" varchar(512) NULL,
	"whatsappBotKey" varchar(255) NULL,
	"whatsappServiceKey" varchar(255) NULL,
	"whatsappServiceNumber" varchar(255) NULL,
	"whatsappServiceUrl" varchar(255) NULL,
	"smsKey" varchar(255) NULL,
	"smsSecret" varchar(255) NULL,
	"smsServiceNumber" varchar(255) NULL,
	"speechKey" varchar(255) NULL,
	"speechEndpoint" varchar(255) NULL,
	"spellcheckerKey" varchar(255) NULL,
	"spellcheckerEndpoint" varchar(255) NULL,
	theme varchar(255) NULL,
	ui varchar(255) NULL,
	kb varchar(255) NULL,
	"nlpAppId" varchar(255) NULL,
	"nlpKey" varchar(255) NULL,
	"nlpEndpoint" varchar(512) NULL,
	"nlpAuthoringKey" varchar(255) NULL,
	"deploymentPaths" varchar(255) NULL,
	"searchHost" varchar(255) NULL,
	"searchKey" varchar(255) NULL,
	"searchIndex" varchar(255) NULL,
	"searchIndexer" varchar(255) NULL,
	"storageUsername" varchar(255) NULL,
	"storagePassword" varchar(255) NULL,
	"storageName" varchar(255) NULL,
	"storageServer" varchar(255) NULL,
	"storageDialect" varchar(255) NULL,
	"storagePath" varchar(255) NULL,
	"adminPass" varchar(255) NULL,
	"searchScore" float8 NULL,
	"nlpScore" float8 NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	params varchar(4000) NULL,
	CONSTRAINT "GuaribasInstance_pkey" PRIMARY KEY ("instanceId")
);


-- public."GuaribasApplications" definition

-- Drop table

-- DROP TABLE "GuaribasApplications";

CREATE TABLE "GuaribasApplications" (
	id serial4 NOT NULL,
	"name" varchar(255) NULL,
	"instanceId" int4 NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	CONSTRAINT "GuaribasApplications_pkey" PRIMARY KEY (id),
	CONSTRAINT "GuaribasApplications_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasGroup" definition

-- Drop table

-- DROP TABLE "GuaribasGroup";

CREATE TABLE "GuaribasGroup" (
	"groupId" serial4 NOT NULL,
	"displayName" varchar(512) NULL,
	"instanceId" int4 NULL,
	CONSTRAINT "GuaribasGroup_pkey" PRIMARY KEY ("groupId"),
	CONSTRAINT "GuaribasGroup_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasLog" definition

-- Drop table

-- DROP TABLE "GuaribasLog";

CREATE TABLE "GuaribasLog" (
	"logId" serial4 NOT NULL,
	message varchar(1024) NULL,
	kind varchar(1) NULL,
	"instanceId" int4 NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	CONSTRAINT "GuaribasLog_pkey" PRIMARY KEY ("logId"),
	CONSTRAINT "GuaribasLog_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasPackage" definition

-- Drop table

-- DROP TABLE "GuaribasPackage";

CREATE TABLE "GuaribasPackage" (
	"packageId" serial4 NOT NULL,
	"packageName" varchar(255) NULL,
	"instanceId" int4 NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	custom varchar(512) NULL,
	CONSTRAINT "GuaribasPackage_pkey" PRIMARY KEY ("packageId"),
	CONSTRAINT "GuaribasPackage_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasQuestionAlternate" definition

-- Drop table

-- DROP TABLE "GuaribasQuestionAlternate";

CREATE TABLE "GuaribasQuestionAlternate" (
	"quickAnswerId" serial4 NOT NULL,
	"questionTyped" varchar(255) NULL,
	"questionText" varchar(255) NULL,
	"instanceId" int4 NULL,
	CONSTRAINT "GuaribasQuestionAlternate_pkey" PRIMARY KEY ("quickAnswerId"),
	CONSTRAINT "GuaribasQuestionAlternate_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasSchedule" definition

-- Drop table

-- DROP TABLE "GuaribasSchedule";

CREATE TABLE "GuaribasSchedule" (
	id serial4 NOT NULL,
	"name" varchar(255) NULL,
	schedule varchar(255) NULL,
	"instanceId" int4 NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	CONSTRAINT "GuaribasSchedule_pkey" PRIMARY KEY (id),
	CONSTRAINT "GuaribasSchedule_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasUser" definition

-- Drop table

-- DROP TABLE "GuaribasUser";

CREATE TABLE "GuaribasUser" (
	"userId" serial4 NOT NULL,
	"displayName" varchar(255) NULL,
	"userSystemId" varchar(255) NULL,
	"userName" varchar(255) NULL,
	"defaultChannel" varchar(255) NULL,
	email varchar(255) NULL,
	locale varchar(5) NULL,
	"instanceId" int4 NULL,
	"agentSystemId" int4 NULL,
	"agentContacted" timestamptz NULL,
	"agentMode" varchar(16) NULL,
	"conversationReference" text NULL,
	"conversationId" int4 NULL,
	"hearOnDialog" varchar(64) NULL,
	params varchar(4000) NULL,
	CONSTRAINT "GuaribasUser_pkey" PRIMARY KEY ("userId"),
	CONSTRAINT "GuaribasUser_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE
);


-- public."GuaribasUserGroup" definition

-- Drop table

-- DROP TABLE "GuaribasUserGroup";

CREATE TABLE "GuaribasUserGroup" (
	id serial4 NOT NULL,
	"userId" int4 NULL,
	"groupId" int4 NULL,
	"instanceId" int4 NULL,
	CONSTRAINT "GuaribasUserGroup_pkey" PRIMARY KEY (id),
	CONSTRAINT "GuaribasUserGroup_groupId_fkey" FOREIGN KEY ("groupId") REFERENCES "GuaribasGroup"("groupId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasUserGroup_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasUserGroup_userId_fkey" FOREIGN KEY ("userId") REFERENCES "GuaribasUser"("userId") ON UPDATE CASCADE
);


-- public."GuaribasAnswer" definition

-- Drop table

-- DROP TABLE "GuaribasAnswer";

CREATE TABLE "GuaribasAnswer" (
	"answerId" serial4 NOT NULL,
	media varchar(512) NULL,
	format varchar(12) NULL,
	"content" text NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	"nextId" int4 NULL,
	"prevId" int4 NULL,
	"instanceId" int4 NULL,
	"packageId" int4 NULL,
	CONSTRAINT "GuaribasAnswer_pkey" PRIMARY KEY ("answerId"),
	CONSTRAINT "GuaribasAnswer_packageId_fkey" FOREIGN KEY ("packageId") REFERENCES "GuaribasPackage"("packageId") ON UPDATE CASCADE
);


-- public."GuaribasQuestion" definition

-- Drop table

-- DROP TABLE "GuaribasQuestion";

CREATE TABLE "GuaribasQuestion" (
	"questionId" serial4 NOT NULL,
	subject1 varchar(64) NULL,
	subject2 varchar(64) NULL,
	subject3 varchar(64) NULL,
	subject4 varchar(64) NULL,
	keywords varchar(1024) NULL,
	"skipIndex" bool NULL,
	"from" varchar(512) NULL,
	"to" varchar(512) NULL,
	"content" text NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	"answerId" int4 NULL,
	"instanceId" int4 NULL,
	"packageId" int4 NULL,
	CONSTRAINT "GuaribasQuestion_pkey" PRIMARY KEY ("questionId"),
	CONSTRAINT "GuaribasQuestion_answerId_fkey" FOREIGN KEY ("answerId") REFERENCES "GuaribasAnswer"("answerId") ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT "GuaribasQuestion_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasQuestion_packageId_fkey" FOREIGN KEY ("packageId") REFERENCES "GuaribasPackage"("packageId") ON UPDATE CASCADE
);


-- public."GuaribasSubject" definition

-- Drop table

-- DROP TABLE "GuaribasSubject";

CREATE TABLE "GuaribasSubject" (
	"subjectId" serial4 NOT NULL,
	"internalId" varchar(255) NULL,
	description varchar(512) NULL,
	"from" varchar(255) NULL,
	"to" varchar(255) NULL,
	"parentSubjectId" int4 NULL,
	"instanceId" int4 NULL,
	"responsibleUserId" int4 NULL,
	"packageId" int4 NULL,
	CONSTRAINT "GuaribasSubject_pkey" PRIMARY KEY ("subjectId"),
	CONSTRAINT "GuaribasSubject_instanceId_fkey" FOREIGN KEY ("instanceId") REFERENCES "GuaribasInstance"("instanceId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasSubject_packageId_fkey" FOREIGN KEY ("packageId") REFERENCES "GuaribasPackage"("packageId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasSubject_parentSubjectId_fkey" FOREIGN KEY ("parentSubjectId") REFERENCES "GuaribasSubject"("subjectId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasSubject_responsibleUserId_fkey" FOREIGN KEY ("responsibleUserId") REFERENCES "GuaribasUser"("userId") ON UPDATE CASCADE
);


-- public."GuaribasConversation" definition

-- Drop table

-- DROP TABLE "GuaribasConversation";

CREATE TABLE "GuaribasConversation" (
	"conversationId" serial4 NOT NULL,
	"instanceId" int4 NULL,
	"startSubjectId" int4 NULL,
	"channelId" int4 NULL,
	"rateDate" timestamptz NULL,
	rate float8 NULL,
	feedback varchar(512) NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	"text" varchar(255) NULL,
	"startedByUserId" int4 NULL,
	CONSTRAINT "GuaribasConversation_pkey" PRIMARY KEY ("conversationId"),
	CONSTRAINT "GuaribasConversation_startSubjectId_fkey" FOREIGN KEY ("startSubjectId") REFERENCES "GuaribasSubject"("subjectId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasConversation_startedByUserId_fkey" FOREIGN KEY ("startedByUserId") REFERENCES "GuaribasUser"("userId") ON UPDATE CASCADE
);


-- public."GuaribasConversationMessage" definition

-- Drop table

-- DROP TABLE "GuaribasConversationMessage";

CREATE TABLE "GuaribasConversationMessage" (
	"conversationMessageId" serial4 NOT NULL,
	"subjectId" int4 NULL,
	"content" text NULL,
	"createdAt" timestamptz NULL,
	"updatedAt" timestamptz NULL,
	"conversationId" int4 NULL,
	"instanceId" int4 NULL,
	"userId" int4 NULL,
	CONSTRAINT "GuaribasConversationMessage_pkey" PRIMARY KEY ("conversationMessageId"),
	CONSTRAINT "GuaribasConversationMessage_conversationId_fkey" FOREIGN KEY ("conversationId") REFERENCES "GuaribasConversation"("conversationId") ON UPDATE CASCADE,
	CONSTRAINT "GuaribasConversationMessage_userId_fkey" FOREIGN KEY ("userId") REFERENCES "GuaribasUser"("userId") ON UPDATE CASCADE
);

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

C. Media Handling:
- SFU WebRTC media servers
- Track multiplexing
- Media processing
- Recording storage

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
- Authorization (zitadel API)
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
- reate environment configuration for everything and ingress to have several server nodes if eeed automatically 
- I NEED FULL CODE SOLUTION IN PROFESSIONAL TESTABLE RUST CODE: if you need split answer in several parts, but provide ENTIRE CODE. Complete working balenced aserver.  IMPORTANTE: Generate the project in a .sh shell script output with cat, of entire code base to be restored, no placeholder neither TODOS. 
- VERY IMPORNTANT: DO NOT put things like  // Add other system routes... you should WRITE ACUTAL CODE
- Need tests for every line of code written.
- single project organized in folders.