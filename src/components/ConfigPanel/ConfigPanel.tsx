import { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { GlassContainer } from "../shared/GlassContainer";
import { useConfig } from "../../hooks/useConfig";
import { useWhisperModel } from "../../hooks/useWhisperModel";
import "./ConfigPanel.css";

const WIDTH = 460;

const QUICK_PROMPTS = [
  {
    label: "Direct Answer",
    prompt:
      "Look at the screen and identify any question or quiz. Reply ONLY with the correct answer, nothing else. No explanation, no reasoning.",
  },
  {
    label: "Answer + Explanation",
    prompt:
      "Look at the screen and identify any question or quiz. Reply with the correct answer followed by a brief explanation of why it is correct.",
  },
  {
    label: "Step by Step",
    prompt:
      "Look at the screen and identify any question or quiz. Solve it step by step, showing your reasoning clearly, then state the final answer.",
  },
  {
    label: "Code Help",
    prompt:
      "Analyze the code visible on screen. Identify any errors, answer any questions about it, or suggest improvements. Be concise.",
  },
];

const MODELS = [
  { value: "gemini-2.0-flash", label: "Gemini 2.0 Flash" },
  { value: "gemini-2.5-flash", label: "Gemini 2.5 Flash" },
  { value: "gemini-2.5-pro", label: "Gemini 2.5 Pro" },
  { value: "gemini-3.1-pro-preview", label: "Gemini 3.1" },
];

const RESPONSE_LANGUAGES = [
  { value: "auto", label: "Auto (match input)" },
  { value: "English", label: "English" },
  { value: "Portuguese", label: "Portuguese" },
  { value: "Spanish", label: "Spanish" },
  { value: "French", label: "French" },
  { value: "German", label: "German" },
  { value: "Italian", label: "Italian" },
  { value: "Japanese", label: "Japanese" },
  { value: "Korean", label: "Korean" },
  { value: "Chinese", label: "Chinese" },
];

const WHISPER_LANGUAGES = [
  { value: "auto", label: "Auto Detect" },
  { value: "en", label: "English" },
  { value: "pt", label: "Portuguese" },
  { value: "es", label: "Spanish" },
  { value: "fr", label: "French" },
  { value: "de", label: "German" },
  { value: "it", label: "Italian" },
  { value: "ja", label: "Japanese" },
  { value: "ko", label: "Korean" },
  { value: "zh", label: "Chinese" },
];

const AUDIO_SOURCES = [
  { value: "both", label: "Mic + System" },
  { value: "mic", label: "Microphone Only" },
  { value: "system", label: "System Audio Only" },
];

const VOCAB_PRESETS = [
  {
    label: "Web Dev",
    seed: "JavaScript, TypeScript, Node.js, React, Next.js, Vue.js, Angular, Svelte, Nuxt, Remix, Vite, webpack, Babel, ESLint, Prettier, npm, yarn, pnpm, HTML, CSS, SASS, SCSS, Tailwind CSS, styled-components, JSX, TSX, component, hook, useState, useEffect, useCallback, useMemo, useRef, useContext, props, state, render, virtual DOM, hydration, server-side rendering, SSR, SSG, ISR, client-side rendering, SPA, single page application, API, REST, RESTful, GraphQL, WebSocket, fetch, axios, CORS, JWT, JSON, XML, middleware, endpoint, route, router, controller, MVC, MVVM, frontend, backend, fullstack, responsive, mobile first, PWA, service worker, localStorage, sessionStorage, cookie, OAuth, authentication, authorization",
  },
  {
    label: "DevOps",
    seed: "Docker, Dockerfile, docker-compose, Kubernetes, K8s, pod, deployment, service, ingress, namespace, helm, AWS, Amazon Web Services, EC2, S3, Lambda, ECS, EKS, RDS, CloudFront, Route 53, IAM, Azure, Google Cloud, GCP, CI/CD, continuous integration, continuous deployment, GitHub Actions, GitLab CI, Jenkins, CircleCI, Terraform, Ansible, Pulumi, CloudFormation, nginx, Apache, load balancer, reverse proxy, SSL, TLS, HTTPS, DNS, CDN, Linux, Ubuntu, Debian, CentOS, SSH, bash, shell script, cron, systemd, container, image, registry, microservice, monolith, scaling, horizontal scaling, auto scaling, monitoring, logging, Grafana, Prometheus, Datadog, ELK, Elasticsearch, Kibana, Logstash, Redis, RabbitMQ, Kafka, message queue, event driven",
  },
  {
    label: "Data / ML",
    seed: "Python, pandas, NumPy, SciPy, matplotlib, seaborn, Jupyter, notebook, TensorFlow, PyTorch, Keras, scikit-learn, XGBoost, LightGBM, machine learning, deep learning, neural network, CNN, RNN, LSTM, transformer, attention, GPT, BERT, LLM, large language model, fine-tuning, transfer learning, embeddings, vector, dataset, dataframe, training, validation, test, inference, epoch, batch, learning rate, loss function, gradient descent, backpropagation, overfitting, underfitting, regularization, dropout, activation function, softmax, ReLU, sigmoid, GPU, CUDA, TPU, modelo, treinamento, classificação, regressão, clustering, NLP, processamento de linguagem natural, tokenização, RAG, retrieval augmented generation, prompt engineering, Hugging Face, ONNX, MLflow, feature engineering, data pipeline, ETL",
  },
  {
    label: "CS / Algo",
    seed: "algoritmo, algorithm, estrutura de dados, data structure, árvore binária, binary tree, binary search tree, BST, árvore AVL, árvore rubro-negra, red-black tree, B-tree, trie, linked list, lista encadeada, doubly linked list, array, vetor, hash map, hash table, tabela hash, grafo, graph, BFS, DFS, busca em largura, busca em profundidade, Dijkstra, Floyd-Warshall, Bellman-Ford, topological sort, minimum spanning tree, Kruskal, Prim, stack, pilha, queue, fila, priority queue, heap, min-heap, max-heap, recursão, recursion, memoization, programação dinâmica, dynamic programming, backtracking, greedy, guloso, divide and conquer, dividir e conquistar, complexidade, Big O, O(n), O(log n), O(n²), sorting, ordenação, quicksort, mergesort, heapsort, bubble sort, insertion sort, binary search, busca binária, two pointers, sliding window, inversão, traversal, in-order, pre-order, post-order, level-order",
  },
  {
    label: "Database",
    seed: "SQL, MySQL, PostgreSQL, Postgres, SQLite, Microsoft SQL Server, Oracle, NoSQL, MongoDB, DynamoDB, Cassandra, CouchDB, Redis, Memcached, database, banco de dados, schema, tabela, table, coluna, column, row, linha, query, consulta, SELECT, INSERT, UPDATE, DELETE, JOIN, INNER JOIN, LEFT JOIN, RIGHT JOIN, WHERE, GROUP BY, ORDER BY, HAVING, INDEX, índice, primary key, chave primária, foreign key, chave estrangeira, constraint, transaction, transação, ACID, normalização, normalization, denormalization, migration, migração, ORM, Prisma, Sequelize, TypeORM, Drizzle, Knex, connection pool, replication, sharding, partitioning, stored procedure, trigger, view, materialized view, EXPLAIN, query plan, N+1",
  },
  {
    label: "Mobile",
    seed: "React Native, Flutter, Dart, Swift, SwiftUI, UIKit, Kotlin, Jetpack Compose, Android, iOS, Xcode, Android Studio, Expo, mobile, app, aplicativo, tela, screen, navigation, navegação, bottom tab, stack navigator, drawer, push notification, notificação, deep link, gesture, touch, scroll, FlatList, ListView, AsyncStorage, SQLite, Realm, Firebase, App Store, Google Play, build, release, debug, emulator, simulador, responsive, adaptive, native module, bridge, Turbo Module, Fabric, Hermes, Metro bundler, CocoaPods, Gradle, APK, IPA, TestFlight",
  },
  {
    label: "Agile",
    seed: "sprint, backlog, product backlog, sprint backlog, user story, história de usuário, task, tarefa, épico, epic, feature, funcionalidade, bug, issue, Jira, Trello, Linear, Kanban, Scrum, daily, stand-up, retrospectiva, retrospective, planning, refinement, grooming, velocity, velocidade, story points, pontos de história, burndown, burnup, release, deploy, deployment, pull request, PR, merge request, MR, code review, revisão de código, branch, commit, push, merge, rebase, cherry-pick, hotfix, rollback, staging, production, produção, homologação, QA, quality assurance, teste, test, unit test, integration test, end-to-end, E2E, acceptance criteria, definition of done, stakeholder, product owner, PO, scrum master, tech lead",
  },
];

type Tab = "general" | "audio" | "stealth" | "shortcuts";

export function ConfigPanel() {
  const { config, updateConfig, autoSaved } = useConfig();
  const { models, downloading, downloadModel } = useWhisperModel();
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const [showKey, setShowKey] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  const resizeToFit = useCallback(async () => {
    if (!panelRef.current) return;
    await new Promise((r) => requestAnimationFrame(r));
    await new Promise((r) => requestAnimationFrame(r));
    const height = panelRef.current.scrollHeight + 2;
    await getCurrentWindow().setSize(new LogicalSize(WIDTH, height));
  }, []);

  useEffect(() => {
    resizeToFit();
  }, [resizeToFit, activeTab]);

  const handleClose = () => {
    getCurrentWindow().hide();
  };

  const handleVocabToggle = (preset: typeof VOCAB_PRESETS[number]) => {
    const isActive = preset.seed
      .split(", ")
      .some((term) => config.vocab_seed.includes(term));

    if (isActive) {
      const presetTerms = new Set(preset.seed.split(", "));
      const remaining = config.vocab_seed
        .split(", ")
        .filter((t) => t.trim() && !presetTerms.has(t.trim()))
        .join(", ");
      updateConfig({ vocab_seed: remaining });
    } else {
      const combined = config.vocab_seed
        ? `${config.vocab_seed}, ${preset.seed}`
        : preset.seed;
      const unique = [
        ...new Set(
          combined
            .split(", ")
            .map((t) => t.trim())
            .filter(Boolean)
        ),
      ];
      updateConfig({ vocab_seed: unique.join(", ") });
    }
  };

  return (
    <GlassContainer ref={panelRef}>
      <div className="titlebar" data-tauri-drag-region>
        <span className="titlebar-title">Phantom Settings</span>
        <div className="titlebar-right">
          {autoSaved && (
            <span className="auto-saved-indicator">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              Saved
            </span>
          )}
          <button className="close-btn" onClick={handleClose}>
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path
                d="M1 1L9 9M9 1L1 9"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>
      </div>

      <div className="tab-bar">
        <button
          className={`tab-btn ${activeTab === "general" ? "active" : ""}`}
          onClick={() => setActiveTab("general")}
        >
          <svg className="tab-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
          General
        </button>
        <button
          className={`tab-btn ${activeTab === "audio" ? "active" : ""}`}
          onClick={() => setActiveTab("audio")}
        >
          <svg className="tab-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="23" />
            <line x1="8" y1="23" x2="16" y2="23" />
          </svg>
          Audio
        </button>
        <button
          className={`tab-btn ${activeTab === "stealth" ? "active" : ""}`}
          onClick={() => setActiveTab("stealth")}
        >
          <svg className="tab-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
          </svg>
          Stealth
        </button>
        <button
          className={`tab-btn ${activeTab === "shortcuts" ? "active" : ""}`}
          onClick={() => setActiveTab("shortcuts")}
        >
          <svg className="tab-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2" y="4" width="20" height="16" rx="2" />
            <line x1="6" y1="8" x2="6.01" y2="8" />
            <line x1="10" y1="8" x2="10.01" y2="8" />
            <line x1="14" y1="8" x2="14.01" y2="8" />
            <line x1="18" y1="8" x2="18.01" y2="8" />
            <line x1="6" y1="12" x2="6.01" y2="12" />
            <line x1="18" y1="12" x2="18.01" y2="12" />
            <line x1="8" y1="16" x2="16" y2="16" />
          </svg>
          Shortcuts
        </button>
      </div>

      <div className="config-content">
        {activeTab === "general" && (
          <>
            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" /></svg>
                API Key
              </label>
              <div className="input-row">
                <input
                  type={showKey ? "text" : "password"}
                  value={config.api_key}
                  onChange={(e) => updateConfig({ api_key: e.target.value })}
                  placeholder="Enter your Gemini API key"
                  spellCheck={false}
                />
                <button
                  className="toggle-btn"
                  onClick={() => setShowKey(!showKey)}
                >
                  {showKey ? "Hide" : "Show"}
                </button>
              </div>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" /></svg>
                Model
              </label>
              <select
                value={config.model}
                onChange={(e) => updateConfig({ model: e.target.value })}
              >
                {MODELS.map((m) => (
                  <option key={m.value} value={m.value}>
                    {m.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" /></svg>
                Response Language
              </label>
              <select
                value={config.response_language}
                onChange={(e) => updateConfig({ response_language: e.target.value })}
              >
                {RESPONSE_LANGUAGES.map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                Prompt
              </label>
              <div className="quick-prompts">
                {QUICK_PROMPTS.map((qp) => (
                  <button
                    key={qp.label}
                    className={`quick-prompt-btn ${config.prompt === qp.prompt ? "active" : ""}`}
                    onClick={() => updateConfig({ prompt: qp.prompt })}
                  >
                    {qp.label}
                  </button>
                ))}
              </div>
              <textarea
                value={config.prompt}
                onChange={(e) => updateConfig({ prompt: e.target.value })}
                rows={3}
                placeholder="Instructions sent with each screenshot"
                spellCheck={false}
              />
            </div>

          </>
        )}

        {activeTab === "audio" && (
          <>
            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="4" y="4" width="16" height="16" rx="2" ry="2" /><rect x="9" y="9" width="6" height="6" /><line x1="9" y1="1" x2="9" y2="4" /><line x1="15" y1="1" x2="15" y2="4" /><line x1="9" y1="20" x2="9" y2="23" /><line x1="15" y1="20" x2="15" y2="23" /><line x1="20" y1="9" x2="23" y2="9" /><line x1="20" y1="14" x2="23" y2="14" /><line x1="1" y1="9" x2="4" y2="9" /><line x1="1" y1="14" x2="4" y2="14" /></svg>
                Whisper Model
              </label>
              <div className="whisper-model-row">
                <select
                  value={config.whisper_model_size}
                  onChange={(e) =>
                    updateConfig({ whisper_model_size: e.target.value })
                  }
                >
                  {models.map((m) => (
                    <option key={m.size} value={m.size}>
                      {m.label} {m.downloaded ? "\u2713" : ""}
                    </option>
                  ))}
                  {models.length === 0 && (
                    <>
                      <option value="tiny">Tiny (~75MB)</option>
                      <option value="base">Base (~142MB)</option>
                      <option value="small">Small (~466MB)</option>
                      <option value="medium">Medium (~1.5GB)</option>
                    </>
                  )}
                </select>
                {models.find(
                  (m) => m.size === config.whisper_model_size && !m.downloaded
                ) && (
                  <button
                    className="toggle-btn"
                    onClick={() => downloadModel(config.whisper_model_size)}
                    disabled={downloading !== null}
                  >
                    {downloading === config.whisper_model_size
                      ? "Downloading..."
                      : "Download"}
                  </button>
                )}
              </div>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" /></svg>
                Transcription Language
              </label>
              <select
                value={config.whisper_language}
                onChange={(e) =>
                  updateConfig({ whisper_language: e.target.value })
                }
              >
                {WHISPER_LANGUAGES.map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" /></svg>
                Audio Source
              </label>
              <select
                value={config.audio_source}
                onChange={(e) =>
                  updateConfig({ audio_source: e.target.value })
                }
              >
                {AUDIO_SOURCES.map((s) => (
                  <option key={s.value} value={s.value}>
                    {s.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" /></svg>
                Vocabulary Context
              </label>
              <div className="quick-prompts">
                {VOCAB_PRESETS.map((vp) => {
                  const isActive = vp.seed
                    .split(", ")
                    .some((term) => config.vocab_seed.includes(term));
                  return (
                    <button
                      key={vp.label}
                      className={`quick-prompt-btn ${isActive ? "active" : ""}`}
                      onClick={() => handleVocabToggle(vp)}
                    >
                      {vp.label}
                    </button>
                  );
                })}
              </div>
              <textarea
                value={config.vocab_seed}
                onChange={(e) => updateConfig({ vocab_seed: e.target.value })}
                rows={3}
                placeholder="Technical terms to help transcription accuracy (e.g., Node.js, árvore binária, API)"
                spellCheck={false}
              />
            </div>
          </>
        )}

        {activeTab === "stealth" && (
          <>
            <div className="stealth-section-label">Screen & Window</div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Stealth Mode</label>
                  <span className="stealth-desc">
                    Hide window from screenshots and recordings
                  </span>
                </div>
                <button
                  className={`stealth-toggle ${config.stealth_mode ? "active" : ""}`}
                  onClick={() =>
                    updateConfig({ stealth_mode: !config.stealth_mode })
                  }
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Passthrough Mode</label>
                  <span className="stealth-desc">
                    Window ignores mouse events to prevent focus detection
                  </span>
                </div>
                <button
                  className={`stealth-toggle ${config.passthrough_mode ? "active" : ""}`}
                  onClick={() =>
                    updateConfig({ passthrough_mode: !config.passthrough_mode })
                  }
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Dodge on Hover</label>
                  <span className="stealth-desc">
                    Move window to opposite corner after 2s hover
                  </span>
                </div>
                <button
                  className={`stealth-toggle ${config.dodge_on_hover ? "active" : ""}`}
                  onClick={() =>
                    updateConfig({ dodge_on_hover: !config.dodge_on_hover })
                  }
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>

            <div className="stealth-section-label">Process</div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" /></svg>
                Process Disguise Name
              </label>
              <input
                type="text"
                value={config.process_disguise_name}
                onChange={(e) =>
                  updateConfig({ process_disguise_name: e.target.value })
                }
                placeholder="e.g. com.apple.accessibility"
                spellCheck={false}
              />
            </div>

            <div className="stealth-section-label">Network</div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Spoof User-Agent</label>
                  <span className="stealth-desc">
                    Disguise API requests as Safari traffic
                  </span>
                </div>
                <button
                  className={`stealth-toggle ${config.spoof_user_agent ? "active" : ""}`}
                  onClick={() =>
                    updateConfig({ spoof_user_agent: !config.spoof_user_agent })
                  }
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Request Jitter</label>
                  <span className="stealth-desc">
                    Random 50-500ms delay before API calls
                  </span>
                </div>
                <button
                  className={`stealth-toggle ${config.network_jitter ? "active" : ""}`}
                  onClick={() =>
                    updateConfig({ network_jitter: !config.network_jitter })
                  }
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>

            <div className="field">
              <label>
                <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2" /><rect x="2" y="14" width="20" height="8" rx="2" ry="2" /><line x1="6" y1="6" x2="6.01" y2="6" /><line x1="6" y1="18" x2="6.01" y2="18" /></svg>
                Proxy URL
              </label>
              <input
                type="text"
                value={config.proxy_url}
                onChange={(e) =>
                  updateConfig({ proxy_url: e.target.value })
                }
                placeholder="socks5://127.0.0.1:1080 (optional)"
                spellCheck={false}
              />
            </div>
          </>
        )}

        {activeTab === "shortcuts" && (
          <div className="field">
            <div className="shortcuts-list">
              <div className="shortcut-row">
                <div className="shortcut-left">
                  <span className="shortcut-icon capture">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="3" width="18" height="18" rx="2" />
                      <circle cx="12" cy="12" r="3" />
                    </svg>
                  </span>
                  <div className="shortcut-info">
                    <span className="shortcut-name">Capture & Analyze</span>
                    <span className="shortcut-desc">Screenshot your screen for AI analysis</span>
                  </div>
                </div>
                <kbd>⌘ ⇧ S</kbd>
              </div>

              <div className="shortcut-row">
                <div className="shortcut-left">
                  <span className="shortcut-icon record">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                      <line x1="12" y1="19" x2="12" y2="23" />
                    </svg>
                  </span>
                  <div className="shortcut-info">
                    <span className="shortcut-name">Toggle Recording</span>
                    <span className="shortcut-desc">Start or stop audio transcription</span>
                  </div>
                </div>
                <kbd>⌘ ⇧ M</kbd>
              </div>

              <div className="shortcut-row">
                <div className="shortcut-left">
                  <span className="shortcut-icon settings">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <circle cx="12" cy="12" r="3" />
                      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
                    </svg>
                  </span>
                  <div className="shortcut-info">
                    <span className="shortcut-name">Toggle Settings</span>
                    <span className="shortcut-desc">Open or close this panel</span>
                  </div>
                </div>
                <kbd>⌘ ⇧ C</kbd>
              </div>
            </div>
          </div>
        )}
      </div>
    </GlassContainer>
  );
}
