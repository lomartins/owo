# owo - Personal Finance Management

**owo** ("money" in Yoruba) is an open, expansible, and reliable personal finance management system. Track, analyze, and automate your financial life with ease.

## Features

- **Multi-interface access**: Web, mobile (iOS/Android), and CLI applications
- **Flexible financial tracking**: Manual transaction entry or automated integration with Open Finance APIs
- **AI-powered insights** (optional): Intelligent financial analysis and recommendations
- **Open Finance integration**: Connect your bank accounts securely for real-time data
- **Offline capability**: Full functionality works offline, with sync when connected
- **Privacy-first**: Self-hosted option available, your data stays yours
- **Cross-platform**: Seamless experience across desktop, web, and mobile

## Tech Stack

### Backend
- **Language**: Rust
- **Purpose**: Core API, financial calculations, data processing

### Frontend/Mobile
- **Kotlin Multiplatform (KMP)**: Shared mobile codebase for iOS and Android
- **Web Interface**: Browser-based access to financial data
- **CLI**: Command-line interface for power users

### Additional
- **Open Finance APIs**: Integration with banking protocols (e.g., Open Banking standards)
- **AI Integration**: Optional LLM integration for financial insights

## Project Structure

This is a monorepo containing:

```
owo/
├── backend/              # Rust backend service
│   └── [Core API, database, business logic]
├── mobile/              # Kotlin Multiplatform (KMP)
│   ├── shared/          # Shared code (iOS/Android)
│   ├── android/         # Android-specific code
│   └── ios/             # iOS-specific code
├── web/                 # Web interface
│   └── [Frontend framework - to be determined]
├── cli/                 # Command-line interface
│   └── [Rust CLI tool]
└── docs/                # Documentation, design, ADRs
```

## Getting Started

_(To be updated once development begins)_

Currently in planning phase. Check back soon for:
- Development environment setup
- Building instructions
- Running tests
- Contributing guidelines

## Planned Features

### Phase 1 (MVP)
- [ ] Manual transaction tracking
- [ ] Basic reports and analytics
- [ ] Web interface
- [ ] CLI tool

### Phase 2
- [ ] Open Finance API integration
- [ ] Mobile apps (iOS/Android via KMP)
- [ ] Real-time account synchronization

### Phase 3
- [ ] AI-powered insights
- [ ] Advanced budgeting and forecasting
- [ ] Multi-user/family accounts

## Philosophy

- **Reliability**: Financial data is critical—expect production-grade code
- **Expansibility**: Plugin architecture and API-first design allow easy extensions
- **User Privacy**: Minimal data collection, encryption by default, self-hosting option
- **Open**: Built on open standards and Open Finance APIs

## Contributing

Contributions welcome! Details coming soon.

## License

_(To be determined)_

## Contact

_(Project repository and contact details will be added)_

---

Built with care to help you manage your money, your way.
