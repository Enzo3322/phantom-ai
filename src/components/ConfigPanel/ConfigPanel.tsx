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

type Tab = "general" | "audio" | "shortcuts";

export function ConfigPanel() {
  const { config, save, saving, saved } = useConfig();
  const { models, downloading, downloadModel } = useWhisperModel();
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("gemini-2.0-flash");
  const [prompt, setPrompt] = useState("");
  const [opacity, setOpacity] = useState(0.85);
  const [stealthMode, setStealthMode] = useState(true);
  const [whisperModelSize, setWhisperModelSize] = useState("base");
  const [whisperLanguage, setWhisperLanguage] = useState("auto");
  const [audioSource, setAudioSource] = useState("both");
  const [vocabSeed, setVocabSeed] = useState("");
  const [showKey, setShowKey] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  const resizeToFit = useCallback(async () => {
    if (!panelRef.current) return;
    await new Promise((r) => requestAnimationFrame(r));
    const height = panelRef.current.scrollHeight;
    await getCurrentWindow().setSize(new LogicalSize(WIDTH, height));
  }, []);

  useEffect(() => {
    resizeToFit();
  }, [resizeToFit, activeTab]);

  useEffect(() => {
    setApiKey(config.api_key);
    setModel(config.model);
    setPrompt(config.prompt);
    setOpacity(config.opacity);
    setStealthMode(config.stealth_mode);
    setWhisperModelSize(config.whisper_model_size);
    setWhisperLanguage(config.whisper_language);
    setAudioSource(config.audio_source);
    setVocabSeed(config.vocab_seed);
  }, [config]);

  const handleOpacityChange = (val: number) => {
    setOpacity(val);
    document.documentElement.style.setProperty("--bg-opacity", String(val));
  };

  const handleSave = () => {
    save({
      api_key: apiKey,
      model,
      prompt,
      opacity,
      stealth_mode: stealthMode,
      whisper_model_size: whisperModelSize,
      whisper_language: whisperLanguage,
      audio_source: audioSource,
      vocab_seed: vocabSeed,
    });
  };

  const handleClose = () => {
    getCurrentWindow().hide();
  };

  return (
    <GlassContainer ref={panelRef}>
      <div className="titlebar" data-tauri-drag-region>
        <span className="titlebar-title">Phantom Settings</span>
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

      <div className="tab-bar">
        <button
          className={`tab-btn ${activeTab === "general" ? "active" : ""}`}
          onClick={() => setActiveTab("general")}
        >
          General
        </button>
        <button
          className={`tab-btn ${activeTab === "audio" ? "active" : ""}`}
          onClick={() => setActiveTab("audio")}
        >
          Audio
        </button>
        <button
          className={`tab-btn ${activeTab === "shortcuts" ? "active" : ""}`}
          onClick={() => setActiveTab("shortcuts")}
        >
          Shortcuts
        </button>
      </div>

      <div className="config-content">
        {activeTab === "general" && (
          <>
            <div className="field">
              <label>API Key</label>
              <div className="input-row">
                <input
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
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
              <label>Model</label>
              <select value={model} onChange={(e) => setModel(e.target.value)}>
                {MODELS.map((m) => (
                  <option key={m.value} value={m.value}>
                    {m.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>Prompt</label>
              <div className="quick-prompts">
                {QUICK_PROMPTS.map((qp) => (
                  <button
                    key={qp.label}
                    className={`quick-prompt-btn ${prompt === qp.prompt ? "active" : ""}`}
                    onClick={() => setPrompt(qp.prompt)}
                  >
                    {qp.label}
                  </button>
                ))}
              </div>
              <textarea
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                rows={3}
                placeholder="Instructions sent with each screenshot"
                spellCheck={false}
              />
            </div>

            <div className="field">
              <label>
                Opacity
                <span className="opacity-value">{Math.round(opacity * 100)}%</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="1"
                step="0.05"
                value={opacity}
                onChange={(e) => handleOpacityChange(Number(e.target.value))}
                className="opacity-slider"
              />
            </div>

            <div className="field">
              <div className="stealth-row">
                <div className="stealth-info">
                  <label>Stealth Mode</label>
                  <span className="stealth-desc">Hide window from screenshots and recordings</span>
                </div>
                <button
                  className={`stealth-toggle ${stealthMode ? "active" : ""}`}
                  onClick={() => setStealthMode(!stealthMode)}
                >
                  <span className="stealth-toggle-knob" />
                </button>
              </div>
            </div>
          </>
        )}

        {activeTab === "audio" && (
          <>
            <div className="field">
              <label>Whisper Model</label>
              <div className="whisper-model-row">
                <select
                  value={whisperModelSize}
                  onChange={(e) => setWhisperModelSize(e.target.value)}
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
                {models.find((m) => m.size === whisperModelSize && !m.downloaded) && (
                  <button
                    className="toggle-btn"
                    onClick={() => downloadModel(whisperModelSize)}
                    disabled={downloading !== null}
                  >
                    {downloading === whisperModelSize ? "Downloading..." : "Download"}
                  </button>
                )}
              </div>
            </div>

            <div className="field">
              <label>Transcription Language</label>
              <select
                value={whisperLanguage}
                onChange={(e) => setWhisperLanguage(e.target.value)}
              >
                {WHISPER_LANGUAGES.map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>Audio Source</label>
              <select
                value={audioSource}
                onChange={(e) => setAudioSource(e.target.value)}
              >
                {AUDIO_SOURCES.map((s) => (
                  <option key={s.value} value={s.value}>
                    {s.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label>Vocabulary Context</label>
              <div className="quick-prompts">
                {VOCAB_PRESETS.map((vp) => {
                  const isActive = vp.seed.split(", ").some((term) =>
                    vocabSeed.includes(term)
                  );
                  return (
                    <button
                      key={vp.label}
                      className={`quick-prompt-btn ${isActive ? "active" : ""}`}
                      onClick={() => {
                        if (isActive) {
                          // Remove this preset's terms
                          const presetTerms = new Set(vp.seed.split(", "));
                          const remaining = vocabSeed
                            .split(", ")
                            .filter((t) => t.trim() && !presetTerms.has(t.trim()))
                            .join(", ");
                          setVocabSeed(remaining);
                        } else {
                          // Add this preset's terms
                          const combined = vocabSeed
                            ? `${vocabSeed}, ${vp.seed}`
                            : vp.seed;
                          // Deduplicate
                          const unique = [...new Set(combined.split(", ").map((t) => t.trim()).filter(Boolean))];
                          setVocabSeed(unique.join(", "));
                        }
                      }}
                    >
                      {vp.label}
                    </button>
                  );
                })}
              </div>
              <textarea
                value={vocabSeed}
                onChange={(e) => setVocabSeed(e.target.value)}
                rows={3}
                placeholder="Technical terms to help transcription accuracy (e.g., Node.js, árvore binária, API)"
                spellCheck={false}
              />
            </div>
          </>
        )}

        {activeTab === "shortcuts" && (
          <div className="field">
            <div className="shortcuts">
              <div className="shortcut-row">
                <span className="shortcut-label">Capture & Analyze</span>
                <kbd>⌘ ⇧ S</kbd>
              </div>
              <div className="shortcut-row">
                <span className="shortcut-label">Toggle Settings</span>
                <kbd>⌘ ⇧ C</kbd>
              </div>
              <div className="shortcut-row">
                <span className="shortcut-label">Toggle Response</span>
                <kbd>⌘ ⇧ A</kbd>
              </div>
              <div className="shortcut-row">
                <span className="shortcut-label">Toggle Recording</span>
                <kbd>⌘ ⇧ M</kbd>
              </div>
              <div className="shortcut-row">
                <span className="shortcut-label">Toggle Transcription</span>
                <kbd>⌘ ⇧ T</kbd>
              </div>
            </div>
          </div>
        )}

        {activeTab !== "shortcuts" && (
          <button
            className="save-btn"
            onClick={handleSave}
            disabled={saving}
          >
            {saved ? "Saved" : saving ? "Saving..." : "Save"}
          </button>
        )}
      </div>
    </GlassContainer>
  );
}
