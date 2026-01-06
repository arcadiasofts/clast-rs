<div align="center">

<img src="banner.png" alt="Clast Banner"/>

<br/>

Rust를 위한 고성능의 확장 가능한 **Content-Defined Chunking (CDC)** 라이브러리입니다.

<br/>

[English](../README.md) │ 한국어

</div>

<br/>

## 개요

**Clast**는 **Content-Defined Chunking (CDC)**을 위해 설계된 모듈화된 라이브러리입니다. 고정된 오프셋(Fixed-Size Chunking, FSC) 대신 콘텐츠를 기반으로 데이터를 가변 크기의 청크로 나누어, 데이터 중복 제거, 백업 시스템 및 효율적인 스토리지 솔루션을 위한 핵심 구성 요소로 활용됩니다.

**Clast**는 다양한 청킹 알고리즘을 지원하도록 설계되어 개발자가 특정 사용 사례에 가장 적합한 전략을 선택할 수 있습니다.

<br/>

## 주요 기능

- **고성능**: 높은 처리량과 낮은 CPU 사용률을 제공하도록 최적화되었습니다.
- **모듈식 아키텍처**: 다양한 CDC 알고리즘을 지원하도록 설계되었습니다.
- **동기 및 비동기 지원**: 동기식 `std::io`와 비동기식 `tokio` 런타임을 모두 지원합니다.

<br/>

## 지원 알고리즘

### FastCDC
*[The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems](https://doi.org/10.1109/TPDS.2020.2984632)* 논문에 기술된 **FastCDC** 알고리즘의 구현체입니다.

다음 5가지 핵심 최적화를 포함합니다:
- 기어(Gear) 기반 롤링 해싱
- 최적화된 해시 판별
- 최소 청크 크기 미만 구간 스킵
- 정규화된 청킹
- 2바이트 롤링 처리

<br/>

## 설치

`cargo`를 사용하여 패키지를 설치하세요:

```bash
cargo add clast
```

### 기능 플래그 (Feature Flags)

**Clast**는 컴파일된 바이너리 크기를 최소화하기 위해 기능 플래그를 사용합니다. 필요한 기능만 선택적으로 활성화할 수 있습니다.

- `fastcdc`: FastCDC 알고리즘 구현을 활성화합니다. (기본값으로 활성화됨)
- `async`: `tokio`를 사용한 비동기 지원을 활성화합니다.

`fastcdc`만 활성화하는 예 (기본 동작):

```bash
cargo add clast
```

`fastcdc`와 비동기 지원을 함께 활성화하는 예:

```bash
cargo add clast --features async
```

또는 `Cargo.toml`에서:

```toml
[dependencies]
clast = { version = "1.0.3", features = ["async"] }
```

<br/>

## 사용법

자세한 사용 예제는 [튜토리얼](tutorials/MENU-ko_kr.md)을 참고해 주세요.

<br/>

## 참고 문헌

* **FastCDC**: Wen Xia et al., "The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems," *IEEE Transactions on Parallel and Distributed Systems*, 2020.

<br/>

## 기여하기

기여를 환영합니다! Pull Request를 자유롭게 열어 주세요.

1. 저장소를 Fork 합니다.
2. Feature 브랜치를 생성합니다 (`git checkout -b feature/new-feature`).
3. 변경 사항을 커밋합니다 (`git commit -m 'Add some feature'`).
4. 브랜치에 푸시합니다 (`git push origin feature/new-feature`).
5. Pull Request를 엽니다.

Pull Request를 열기 전에 `cargo fmt`와 `cargo test`를 실행했는지 확인해 주세요.

<br/>

## 라이선스

MIT © Arcadia Softs. 자세한 내용은 [LICENSE](../LICENSE)를 참조하세요.

