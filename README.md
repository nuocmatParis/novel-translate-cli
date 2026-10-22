# novel-translate-cli

CLI và TUI dịch light novel **Nhật → Việt**, nhận EPUB và xuất EPUB. Mỗi workspace quản lý một bộ truyện, gồm nhiều tập dùng chung quy ước, thuật ngữ và lịch sử xưng hô theo thời điểm của truyện.

Luồng chính: **nhập EPUB → dịch → review/sửa → xử lý chữ trong ảnh → kiểm tra → xuất EPUB**. Hai chương logic mỗi đợt; hệ thống tự chạy tiếp và lưu kết quả theo khối. Một chương dài có thể cần nhiều request API.

Phần lõi có kiểm thử offline và kiểm tra EPUB bằng EPUBCheck. Chất lượng dịch JP–VN, chi phí toàn tập và tính liên tục xuyên tập vẫn cần nghiệm thu bằng model thật. EPUB hợp lệ không đồng nghĩa với bản dịch đúng nghĩa 100%.

## 1. Cài đặt

| Thành phần | Khi nào cần |
| --- | --- |
| Go **1.25.5+** | Chạy từ mã nguồn |
| API key và model phù hợp | Dịch, review và AI đề xuất quy ước |
| Java và EPUBCheck | Xuất EPUB hoàn chỉnh có kiểm chứng định dạng |
| Bốn font Liberation Serif | Tùy chọn, để nhúng font vào EPUB |

Import, xem tiến độ, xuất ảnh và mở TUI không cần API key. Không cần Python, Docker hay Ollama để chạy CLI với API cloud. Các script nghiệm thu bổ sung dùng Python 3; chạy race test cần cgo và C compiler.

Chạy từ source trên Windows, trong thư mục repo:

```powershell
.\scripts\run.ps1

# Truyền lệnh CLI qua script
.\scripts\run.ps1 --help
```

Linux/macOS:

```sh
# Trong thư mục repo; biên dịch source hiện tại rồi mở TUI
sh scripts/run.sh

# Hoặc dùng Make
make run

# Truyền lệnh CLI qua script
sh scripts/run.sh status --workspace ./workspace/bo-truyen
```

Các script dùng `go run`: mỗi lần mở đều biên dịch từ source hiện tại, Go tự dùng cache cho phần không đổi. Không cần giữ `.exe` trong thư mục gốc. Mặc định mở TUI; tham số truyền sau script được chuyển cho CLI. Script chạy tại thư mục repo, nên đường dẫn tương đối được tính từ đó.

Có thể chạy trực tiếp, không qua script:

```powershell
go run -trimpath -ldflags="-s -w" ./cmd/novel-translate-cli tui
```

Nếu cần binary riêng để đóng gói hoặc dùng mã thoát CLI chính xác, `make build` đặt file duy nhất trong `bin/`. Không có Make trên Windows thì dùng `go build -trimpath -ldflags="-s -w" -o bin/novel-translate-cli.exe ./cmd/novel-translate-cli`. Khi chạy qua `go run`, lỗi của ứng dụng được trả về shell với mã `1`; mã gốc vẫn hiện trong thông báo `exit status`.

Cài vào thư mục người dùng trên Linux/macOS:

```sh
sh scripts/install.sh
"$HOME/.local/bin/novel-translate-cli" tui
```

Script build checkout hiện tại vào `~/.local/bin`; biến `NOVEL_TRANSLATE_INSTALL_DIR` đổi thư mục đích. Gọi bằng đường dẫn trên không cần chỉnh `PATH`. API key lưu trong TUI hoặc `.env` nên không cần `export` lại mỗi lần mở.

Workflow CI tạo artifacts Windows/Linux. Workflow đóng gói được kích hoạt thủ công và tạo snapshot cho Windows/Linux/macOS, amd64/arm64; không tự phát hành GitHub Release. Build chéo chưa chứng minh chạy thực tế trên mọi hệ điều hành.

Các ví dụ sau dùng **PowerShell** và script chạy source trong thư mục repo. Trên Linux/macOS, thay `.\scripts\run.ps1` bằng `sh scripts/run.sh`. Đường dẫn và tên model trong ví dụ cần thay theo máy/dịch vụ của bạn.

## 2. Mở TUI

```powershell
.\scripts\run.ps1
```

Chạy script không có đối số cũng mở TUI; `--help` xem phím tắt. Nếu thư mục hiện tại có `workspace.yaml`, ứng dụng mở workspace đó. Nếu không, màn **Khởi đầu** hiển thị workspace gần đây và các workspace trong `workspace/`; **o** mở bằng đường dẫn, **n** tạo mới chỉ từ tên bộ truyện. CLI vẫn cho phép chỉ định trực tiếp:

```powershell
.\scripts\run.ps1 tui --workspace .\workspace\bo-truyen --volume vol01
```

Alias: `--tui`. Trong workspace, **a** đăng ký tập bằng ID, thứ tự, đường dẫn EPUB và tập trước; **Ctrl+O** mở file picker ở trường đường dẫn; **i** import snapshot nguồn. Cấu hình nằm trong màn **Cấu hình**: provider/model/key, giới hạn request, thư mục xuất, EPUBCheck và font. Key được che khi nhập; API key được lưu trong `.env` của workspace.

Từ **Bộ truyện & nội dung**, chọn tập/chương và nhấn **Enter** tại chế độ chạy để bắt đầu dịch hoặc review. Tác vụ model chỉ bắt đầu sau hành động này; mở màn, xem dữ liệu và đổi focus không gọi model. **p** pause/resume dịch/review tại ranh giới khối; **Ctrl+C** yêu cầu dừng. **Bản dịch** có so sánh JP–VI, sửa tay bằng **Ctrl+S**, nguồn/Ruby, lịch sử generation và ghi chú riêng theo khối. Ghi chú gắn với hash nguồn, được lưu riêng và không đưa vào target hoặc prompt; **Ctrl+S** lưu loại nội dung ở tab đang mở. **Reviewer** hiển thị issue, bằng chứng, ruby, signals và cho phép review lại, sửa tay hoặc giữ bản kèm lý do. **Kiểm tra & EPUB** hỗ trợ đánh giá occurrence ảnh, xuất preview và xuất Final sau khi các cổng trước đạt; Final chỉ ghi file sau readback và EPUBCheck thành công.

TUI dùng dark mode nhiều khung (background `#07141B`, panel `#0B1B26`, border `#344A5E`, accent `#1677FF`), co giãn theo terminal. Dưới `80×24`, giao diện giữ hướng dẫn dừng/thoát/trợ giúp và yêu cầu mở rộng terminal.

Các màn hình chính:
- **S00. Mở / Tạo workspace**: Đọc metadata các bộ đã có thành danh sách để chọn; tạo bộ mới chỉ bằng tên.
- **S01. Dashboard**: Thống kê tổng quan, các tập, các blocking issues, tiến độ đối chiếu.
- **S02. Bộ truyện & nội dung**: Xem tập/chương và chọn toàn tập, chương đã đánh dấu hoặc review lại.
- **S03. Tác vụ đang chạy**: Theo dõi sự kiện dịch/review/import/export; pause tại ranh giới khối chỉ dành cho dịch/review.
- **S04. Bản dịch (Editor)**: Cây khối nguồn, JP–VI, sửa tay tạo revision mới (`Ctrl+S`), nguồn/Ruby, lịch sử generation và ghi chú riêng gắn với hash nguồn.
- **S05. Reviewer**: Hàng đợi, issue và bằng chứng, ruby, signals, quyết định giữ có lý do, review lại và mở editor.
- **S06. Skills & thuật ngữ**: Quy ước chung/riêng, rules, candidates và dữ liệu research đã lưu.
- **S07. Ngữ cảnh & trí nhớ**: Cặp xưng hô có hướng, mốc truyện, bằng chứng ngữ cảnh.
- **S08. Kiểm tra & xuất EPUB**: Checklist từ dữ liệu nguồn/generation, đánh giá ảnh, preview/final và kết quả readback/EPUBCheck.
- **S09. Cấu hình**: Provider, model, API key (được che và lưu trong `.env`), giới hạn tác vụ và công cụ kiểm tra.
- **S10. Báo cáo & usage**: Thống kê chi phí USD, input/output tokens và số lượng requests (phân biệt rõ requests chưa xác định đơn giá).

| Phím | Thao tác |
| --- | --- |
| Tab | Chuyển vùng focus giữa Sidebar menu và nội dung các panel |
| ↑ / ↓ hoặc j / k | Chọn dòng / di chuyển trong danh sách bảng |
| ← / → | Chuyển tab / đóng mở nhánh cây |
| Enter | Mở mục đang chọn; thực hiện hành động |
| Space | Chọn / bỏ chọn chương trong màn chọn phạm vi |
| Ctrl+S | Lưu bản dịch, ghi chú riêng hoặc cấu hình theo màn/tab đang mở |
| Ctrl+O | Chọn file hoặc thư mục tại trường đường dẫn |
| p | Tạm dừng / tiếp tục tác vụ tại ranh giới khối kế tiếp (ở màn Tác vụ) |
| k | Giữ bản dịch hiện tại và nhập lý do bắt buộc (ở màn Reviewer) |
| r | Yêu cầu reviewer chấm lại khối được chọn (ở màn Reviewer) |
| e | Chuyển sang màn hình sửa tay cho khối đang xem (ở màn Reviewer) |
| Esc | Hủy hộp thoại / quay lại màn trước (tác vụ chạy nền vẫn tiếp tục) |
| Ctrl+C | Dừng khẩn cấp tác vụ đang chạy và giải phóng khóa writer |
| q | Thoát ứng dụng khi ở trạng thái rảnh |
| ? | Mở bảng trợ giúp phím tắt theo ngữ cảnh màn hình hiện tại |

## 3. Tạo workspace và nhập tập đầu

```powershell
$workspacePath = Join-Path $PWD 'workspace/bo-truyen'
.\scripts\run.ps1 init $workspacePath --series bo-truyen --title "Tên bộ truyện"
.\scripts\run.ps1 volume add --workspace $workspacePath --id vol01 --order 1 --source "D:/Sach/Tap01_JP.epub"
.\scripts\run.ps1 import --workspace $workspacePath --volume vol01
.\scripts\run.ps1 status --workspace $workspacePath
```

Truyền trực tiếp file EPUB, không cần giải nén. `import` lập kiểm kê nguồn và mapping các khối văn bản, chương, ảnh, thuộc tính và chú thích. Thứ tự đọc lấy từ spine/anchors, không theo tên file. Nội dung chưa hỗ trợ được ghi rõ và có thể chặn xuất hoàn chỉnh.

Thêm tập tiếp theo vào cùng workspace:

```powershell
.\scripts\run.ps1 volume add --workspace $workspacePath --id vol02 --order 2 --previous vol01 --source "D:/Sach/Tap02_JP.epub"
```

`--previous` nối ngữ cảnh đã được chấp nhận của tập trước. Dữ liệu xưng hô được giới hạn theo thời điểm/cảnh; không lấy trạng thái cuối truyện áp cho hồi tưởng.

Sau các bước import/dịch/export, workspace có dạng:

```text
workspace/bo-truyen/
├── workspace.yaml
├── .env                           # Key cục bộ nếu thiết lập model qua TUI
├── Skills/
│   ├── SKILL.md                    # Quy ước riêng của bộ truyện
│   ├── imported/                  # Ghi chú tham khảo đã import
│   └── rules.json                 # Xuất hiện khi có quy tắc được lưu
├── vol01_translation/
│   ├── chapters/                  # Bản thảo Markdown có unit markers
│   └── images/source/             # Ảnh lấy ra bằng image export
├── vol02_translation/
├── Output/                        # EPUB và report sau export
└── .translation_work/             # Snapshots, generations, jobs, evidence, usage
```

File nguồn ngoài workspace chỉ được đọc. Đường dẫn tương đối trong `workspace.yaml`, đường dẫn file của `image assess` và file provider được tính từ workspace, không từ thư mục terminal. `--source` của `volume add` được chuyển thành đường dẫn tuyệt đối khi đăng ký.

## 4. Cấu hình model

Cách nhanh: mở **Cấu hình → Provider & Model**, không cần lệnh đặt biến môi trường. Ô API OpenAI chỉ áp dụng cho giao thức `openai`; Gemini/Anthropic dùng giao thức native. Để trống key khi lưu lại sẽ dùng key hiện có. Nếu đổi dịch vụ, nhập key tương ứng của dịch vụ mới.

Nếu muốn cấu hình nâng cao bằng file, chọn một mẫu:

| Dịch vụ / giao thức | File mẫu | Lưu ý |
| --- | --- | --- |
| OpenAI-compatible | [config.example.jsonc](config.example.jsonc) | Điền `base_url`, model; `api` là `chat` hoặc `responses` |
| Gemini native | [config.gemini.example.jsonc](config.gemini.example.jsonc) | `type: gemini`; không thêm `api: chat` |
| Anthropic native | [config.anthropic.example.jsonc](config.anthropic.example.jsonc) | `type: anthropic`; không thêm `api: chat` |

Ví dụ sao chép mẫu OpenAI-compatible vào thư mục cấu hình riêng:

```powershell
New-Item -ItemType Directory -Force .\config | Out-Null
Copy-Item .\config.example.jsonc .\config\provider.jsonc
```

Mở file vừa sao chép, thay **tất cả** placeholder tên model/endpoint theo dịch vụ và quyền truy cập của API key. Đặt `context_window` theo model nếu cần; khi không biết, runtime dùng registry hoặc mặc định bảo thủ. Mẫu chưa điền chưa phải cấu hình chạy được.

Key có thể lưu trong `.env` cạnh `workspace.yaml`, cạnh file provider, hoặc trong thư mục chạy ứng dụng. Tạo từ mẫu rồi mở file để điền key của bạn:

```powershell
Copy-Item .\.env.example .\workspace\bo-truyen\.env
```

Linux/macOS:

```sh
cp .env.example ./workspace/bo-truyen/.env
```

Nội dung `.env` dùng tên biến trùng với `api_key_env` trong file provider:

```dotenv
NOVEL_TRANSLATE_API_KEY=YOUR_API_KEY
```

Ứng dụng tự đọc `.env` mỗi lần cần cấu hình model, không cần chạy `source`, `export` hoặc lệnh PowerShell ở mỗi phiên. Thứ tự ưu tiên cho từng key: **biến môi trường có giá trị → `.env` workspace → `.env` cạnh provider → `.env` thư mục đang chạy**. Giá trị môi trường rỗng cho phép đọc file. Các file này không được nạp vào môi trường chung của tiến trình, nên mở workspace khác không kế thừa key vừa đọc. Nếu terminal đang đặt key cùng tên, key đó vẫn thắng key mới lưu trong TUI; bỏ thiết lập cũ hoặc mở terminal mới để dùng key trong file.

TUI giữ các biến khác khi cập nhật `.env`, nhưng có thể định dạng lại file và bỏ comment. Key được lưu dạng văn bản trong file cục bộ; file do ứng dụng tạo dùng quyền `0600` trên hệ thống POSIX. Repo và Docker build context đã loại `.env`/`.env.*`, trừ mẫu `.env.example` được theo dõi trong Git.

Trong TUI, mở **Cấu hình → Provider & Model** để nhập provider/model/key. Cấu hình provider được lưu trong workspace. Nếu cần nhiều provider, roles hoặc fallback riêng, giữ cấu hình JSON hiện có và trỏ `provider_config` ở cấp ngoài cùng trong `workspace.yaml`:

```yaml
provider_config: ../../config/provider.jsonc
```

Đường dẫn trên đúng khi dùng cấu trúc ví dụ trong repo. Có thể dùng đường dẫn tuyệt đối. Nếu bỏ `provider_config`, CLI tìm `~/.novel-translate/config.json`. Không lưu key trong sách/skill hoặc commit cấu hình chứa key. File JSON/JSONC hỗ trợ UTF-8 và UTF-8 có BOM; không dùng UTF-16.

Ba role là `translation` (dịch), `translation_review` (đối chiếu) và `translation_research` (AI đề xuất quy ước). Role không khai báo dùng model mặc định; có thể cấu hình model/fallback khác nhau theo mẫu. Research web do research adapter thực hiện, không tự có chỉ vì chọn một model.

Endpoint nội bộ không xác thực có thể dùng `allow_no_api_key: true` và bỏ `api_key_env`. Điều này không thay thế API key của dịch vụ yêu cầu xác thực.

### Giới hạn khi chạy thử

Trong TUI, mở **Cấu hình → Giới hạn tác vụ** để đổi limits. Các tùy chọn chi tiết có thể sửa trong phần `run` đã có của `workspace.yaml`, không tạo thêm phần `run` trùng lặp:

```yaml
run:
  chapters_per_batch: 2
  auto_continue: true
  max_contract_corrections: 3
  max_repair_rounds: 3
  max_transport_retries: 3
  request_timeout_seconds: 180
  max_input_bytes: 16000
  max_output_tokens: 12000
  max_requests: 20
```

`max_requests` tính cộng dồn trên **toàn workspace**, gồm retries/fallbacks; `0` là không giới hạn. Khi tăng giới hạn rồi `resume`, kết quả còn hợp lệ được dùng lại. Hai chương mỗi đợt không có nghĩa mỗi lần gọi API chứa hai chương.

`budget.book_usd` nằm trong file provider; `0` tắt giới hạn USD. Khi bật giới hạn mà giá request trước không xác định, runtime dừng thay vì đoán. Chi phí còn phụ thuộc token ngữ cảnh/quy ước, review, sửa lỗi, suy luận và retries; cần đo trên mẫu thật, không có giá cố định cho một tập.

## 5. Quy ước, thuật ngữ và xưng hô

Skill được chia thành **thư viện chung cho mọi bộ** và **quy ước riêng trong mỗi workspace**. Thư viện chung mặc định nằm trong thư mục người dùng:

```text
~/.novel-translate/
├── skills.json                    # Lưu thư mục chung đã chọn
└── skills/
    ├── jp-vn-translation/SKILL.md
    ├── dialogue-and-honorifics/SKILL.md
    └── translation-review/SKILL.md

workspace/
├── rezero/Skills/SKILL.md          # Chỉ quy ước riêng của Re:Zero
└── bo-truyen-khac/Skills/SKILL.md   # Chỉ quy ước riêng của bộ khác
```

Trên Windows, `~` là thư mục người dùng, chẳng hạn `C:/Users/Phun`. Thư viện skill chung được nạp theo cấu hình hiện có; màn **Skills & thuật ngữ** hiển thị nội dung đang dùng. Thay đổi `skill_libraries` trong `workspace.yaml` nếu cần trỏ sang thư viện khác.

Trong màn **Skills & thuật ngữ**, nhấn **Space** để bật/tắt skill chung cho workspace, **Ctrl+S** để lưu, **e** để sửa skill đang chọn và **l** để chọn thư mục thư viện chung. Sửa skill chung có thể ảnh hưởng các workspace đang bật skill đó. Có thể thêm module theo dạng `<id>/SKILL.md` hoặc `<id>.md`, với `id` trong frontmatter trùng tên; sau đó bật ID trong TUI. Chọn đồng thời hai định nghĩa cùng ID sẽ báo lỗi.

Workspace mới không sao chép skill chung vào `Skills/core/`. Các bản sao cũ ở đó được giữ nguyên nhưng không còn là nguồn mặc định. Khi nội dung mặc định giống bản cũ, thay vị trí file không làm mất hiệu lực của bản dịch đã lưu. Khi nội dung thực sự thay đổi, hệ thống đánh dấu phần bị ảnh hưởng để đối chiếu bằng `retranslate plan/apply`. Cấu hình nâng cao `skill_libraries` có sẵn vẫn chỉ tới các thư viện đã khai báo; các đường dẫn đó được đọc trực tiếp, không sao chép vào workspace.

**Quy ước riêng của bộ truyện luôn ưu tiên khi xung đột với skill chung.** Quy ước Re:Zero giữ hậu tố theo nguồn vì vậy vẫn thắng hướng dẫn chung về hậu tố. Thuật ngữ, ghi chú, rules và lịch sử xưng hô riêng vẫn ở trong workspace của bộ, không được đưa vào thư viện chung.

Màn **Skills & thuật ngữ** hiển thị dữ liệu đang dùng và cho sửa skill chung hoặc `Skills/SKILL.md` với kiểm tra hash trước khi ghi. File mặc định có priority 100; quy ước riêng cần nêu rõ phạm vi và ưu tiên nguồn Nhật. Ví dụ:

```markdown
---
id: bo-truyen
priority: 100
---
# Quy ước bộ truyện
- Giữ hậu tố kính ngữ theo nguyên văn.
- Không tự thêm hành động, cảm xúc hoặc thông tin không có trong nguồn.
- Tên nhân vật và thuật ngữ theo bảng đã xác nhận của bộ truyện.
- Xưng hô phụ thuộc người nói → người nghe và thời điểm của cảnh.
```

Tránh đưa tình tiết tương lai vào chỉ dẫn chung. Quy tắc chung, quy ước riêng và dữ liệu tham khảo có scope/version; xung đột quy tắc đã chốt cần được giải quyết rõ ràng.

```powershell
.\scripts\run.ps1 skill list --workspace $workspacePath
.\scripts\run.ps1 skill import --workspace $workspacePath --source "D:/GhiChu/quy-uoc.md"
.\scripts\run.ps1 skill import --workspace $workspacePath --source "D:/GhiChu/tap26-27.md" --from-volume 26 --until-volume 28
```

`until-volume` là cận trên **không bao gồm**. Import giữ văn bản, hash và nguồn; không biến mọi ghi chú thành quy tắc đã xác nhận. Đoạn chưa rõ phạm vi được giữ riêng. AI tạo quy ước dùng bản dịch đã được chấp nhận; nếu thiếu cơ sở, mục mới vẫn là candidate/conflict.

```powershell
.\scripts\run.ps1 skill research --workspace $workspacePath --volume vol01 --term "Thuật ngữ có trong nguồn"
.\scripts\run.ps1 skill generate --workspace $workspacePath --volume vol01
.\scripts\run.ps1 skill resolve --workspace $workspacePath --id RULE_ID --decision confirm --reason "Căn cứ xác nhận"
```

Research cần khai báo `research_sources` trong YAML: `api` hỗ trợ `fetch` (URL cụ thể), `mediawiki` (site dùng API MediaWiki) hoặc `json` (endpoint tìm kiếm trả danh sách URL). Mỗi nguồn có `id`, `base_url`, `edition`, `kind`. Chỉ fetch/redirect trong origin đã cấu hình; lưu URL, hash nội dung và thời điểm. Nguồn official/fandom là bằng chứng tham khảo, không thay nguyên tác và không được tạo URL giả. Dùng `retranslate plan` sau khi đổi quy ước ảnh hưởng bản dịch cũ.

## 6. Dịch, xem tiến độ và sửa tay

```powershell
.\scripts\run.ps1 translate --workspace $workspacePath --volume vol01
.\scripts\run.ps1 status --workspace $workspacePath --json
.\scripts\run.ps1 resume --workspace $workspacePath --volume vol01
```

Luồng tự dịch → review → sửa trong giới hạn rồi tiếp tục; dừng nếu còn vấn đề chặn, xung đột hoặc chạm giới hạn. Ctrl+C yêu cầu dừng. `resume` dùng lại kết quả hợp lệ đã lưu; request đã gửi nhưng chưa lưu kết quả có thể phải gọi lại và tính phí lại.

Trong TUI, mở **Bản dịch**, chọn khối trong cây nguồn rồi chuyển sang tab **Bản dịch (Sửa)**. Nguồn Nhật hiển thị ở tab so sánh; **Ctrl+S** lưu revision và cập nhật Markdown cùng lúc, không cần chạy `sync`. Tab **Ghi chú** lưu riêng theo khối và hash nguồn trong `volumes/<id>/notes.json`; ghi chú không đi vào target hoặc prompt, và được đánh dấu cần đối chiếu khi nguồn đổi. Inline tokens `⟦…⟧` phải giữ nguyên; bản sửa làm mất token hoặc đổi nguồn sẽ bị từ chối. Bản sửa và ngữ cảnh phía sau cần được review lại.

Nếu sửa bằng trình soạn thảo ngoài, chỉ đổi văn bản giữa `<!-- unit:ID -->` và `<!-- /unit -->` trong `vol01_translation/chapters/*.md`. Giữ nguyên markers, thứ tự và inline tokens `⟦…⟧`. Sau đó:

```powershell
.\scripts\run.ps1 sync --workspace $workspacePath --volume vol01
.\scripts\run.ps1 review --workspace $workspacePath --volume vol01 --chapter CHAPTER_ID
```

`CHAPTER_ID` lấy từ `status --json`. Bản sửa tay được lưu thành revision riêng; cần review hoặc quyết định giữ có lý do. Khi đổi nguồn, quy ước hoặc ngữ cảnh:

```powershell
.\scripts\run.ps1 retranslate plan --workspace $workspacePath --volume vol01
.\scripts\run.ps1 retranslate apply --workspace $workspacePath --plan PLAN_ID
```

Đọc plan trước khi apply. `PLAN_ID` do lệnh plan trả về. File EPUB nguồn thay đổi phải qua `source plan` / `source apply`. `restore --generation GENERATION_ID` khôi phục một generation đã lưu; không ghi đè bản sửa tay chưa sync. `scene set/clear` dành cho mốc thời gian/hồi tưởng, cần source quote và lý do; xem `--help` để biết flags.

## 7. Xử lý chữ trong ảnh

Xem danh sách và lấy ảnh ra workspace để kiểm tra/edit:

```powershell
.\scripts\run.ps1 image list --workspace $workspacePath --volume vol01
.\scripts\run.ps1 image export --workspace $workspacePath --volume vol01
```

`image export` trả JSON gồm occurrence ID và đường dẫn ảnh; mặc định ghi vào `vol01_translation/images/source/`. Có thể đổi bằng `--directory` tương đối với workspace. Lệnh không ghi đè file đã được bạn sửa; nên lưu ảnh edit ở thư mục riêng.

Mỗi lần xuất hiện của ảnh cần một quyết định có lý do:

| Disposition | Ý nghĩa |
| --- | --- |
| `no-text` | Đã xem và xác nhận không có chữ cần dịch |
| `keep-original` | Chủ động giữ chữ/ảnh gốc, ví dụ logo/chữ ký; không được coi là đã dịch phần chữ đó |
| `replace` | Dùng ảnh Việt đã tự edit, giữ định dạng tài nguyên gốc |
| `transcript` | Giữ ảnh và thêm bản dịch chữ sau khối nội dung chứa ảnh |

```powershell
.\scripts\run.ps1 image assess --workspace $workspacePath --volume vol01 --id IMAGE_ID --disposition no-text --reason "Đã xem: tranh không có chữ"
.\scripts\run.ps1 image assess --workspace $workspacePath --volume vol01 --id IMAGE_ID --disposition replace --file "images/edited/anh-viet.jpg" --reason "Đã đối chiếu ảnh edit với nguồn"
.\scripts\run.ps1 image assess --workspace $workspacePath --volume vol01 --id IMAGE_ID --disposition transcript --transcript-file "images/anh-viet.txt" --reason "Đã đối chiếu bản dịch chữ trong ảnh"
```

Thay `IMAGE_ID` bằng occurrence ID thật, và chọn một quyết định phù hợp cho mỗi ảnh. File transcript phải là UTF-8; có thể dùng `--transcript "Bản dịch ngắn"` thay file. Trong TUI, mở **Kiểm tra & EPUB**, chọn occurrence ảnh, nhấn **a**, nhập disposition/lý do và đường dẫn thay thế hoặc transcript rồi lưu.

Ảnh thay thế được kiểm tra hash khi verify/export; sửa file sau khi xác nhận thì phải assess lại. Các vị trí dùng chung tài nguyên phải thống nhất dùng ảnh gốc hoặc cùng bản thay thế. Chú thích ảnh độc lập có thể khác nhau theo vị trí. Readback kiểm tra phần chú thích, thứ tự ảnh và bytes tài nguyên.

Chưa có OCR, tự dịch hay tự xóa/chèn chữ vào tranh. Bạn đọc/dịch/kiểm tra và edit ảnh bằng công cụ riêng, rồi đưa kết quả vào workspace.

## 8. Kiểm tra và xuất EPUB

Tải và giải nén **toàn bộ** [EPUBCheck distribution](https://github.com/w3c/epubcheck/releases), giữ JAR cùng thư mục `lib`. Kiểm tra Java và checker:

```powershell
java -version
java -jar "D:/Tools/epubcheck/epubcheck.jar" --version
```

Trong TUI, mở **Cấu hình → Xuất & EPUBCheck** để đặt thư mục xuất, executable/JAR EPUBCheck và thư mục font. Hoặc sửa phần `output` hiện có trong `workspace.yaml`:

```yaml
output:
  directory: Output
  template: sample-vi
  epubcheck: D:/Tools/epubcheck/epubcheck.jar
  # Tùy chọn:
  # font_directory: D:/Tools/fonts/liberation
```

`epubcheck` cũng nhận đường dẫn executable. Thư mục font nếu khai báo phải có `LiberationSerif-Regular.ttf`, `LiberationSerif-Bold.ttf`, `LiberationSerif-Italic.ttf`, `LiberationSerif-BoldItalic.ttf`; nếu bỏ qua thì không nhúng font. Hiện chỉ có template `sample-vi`, giữ cấu trúc nguồn và áp kiểu đọc ngang/LTR.

```powershell
.\scripts\run.ps1 verify --workspace $workspacePath --volume vol01
.\scripts\run.ps1 export --workspace $workspacePath --volume vol01 --preview
.\scripts\run.ps1 export --workspace $workspacePath --volume vol01
```

- `verify`: kiểm tra độ phủ, revision/ngữ cảnh còn hợp lệ và quyết định ảnh; chưa dựng EPUB hay chạy EPUBCheck.
- `--preview`: xuất bản xem trước có nhãn; có thể còn thiếu dịch hoặc checker chưa pass. Đọc report đi kèm, không coi là bản hoàn chỉnh.
- Export hoàn chỉnh: yêu cầu bản dịch đã đối chiếu còn hợp lệ, không còn vấn đề chặn và mọi ảnh có quyết định hợp lệ; sau đó dựng file tạm, mở lại đối chiếu rồi chạy EPUBCheck thật trước khi đưa vào `Output/`.

Đầu ra mặc định: `Output/vol01_VI.epub` hoặc `Output/vol01_VI.preview.epub`, kèm `.report.json`. Có thể ghi đè bản xuất cùng tên sau khi kiểm tra đạt. Report phân biệt nghĩa/độ phủ, cấu trúc và định dạng. Vẫn cần mở trên reader bạn dùng để kiểm tra trình bày.

Xuất một phạm vi chương dùng `--from CHAPTER_ID --to CHAPTER_ID`, bao gồm cả hai đầu; có thể kết hợp `--preview`. Chương có thể gồm nhiều XHTML/trang ảnh. Phạm vi hoàn chỉnh không có nghĩa toàn tập đã hoàn chỉnh.

## 9. Kiểm thử và đóng gói

```powershell
go vet ./...
go test -buildvcs=false -count=1 ./...
git diff --check
```

Suite mặc định dùng fixtures tổng hợp, model giả và HTTP server cục bộ. Khi `NOVEL_TRANSLATE_LIVE_PROVIDER` chưa đặt, test model thật được skip.

Linux/macOS có thể dùng `make test` và `make vet`. Các test giao diện kiểm tra form, lưu key và `.env`, sửa bản dịch/quy ước, bảo vệ xung đột và mở TUI bằng input giả; vẫn cần thử trực tiếp trên terminal bạn dùng.

Chạy thêm kiểm tra EPUB hoàn chỉnh bằng Java/checker thật, với model giả:

```powershell
$env:NOVEL_TRANSLATE_TEST_EPUBCHECK = 'D:/Tools/epubcheck/epubcheck.jar'
go test -buildvcs=false -count=1 -run 'TestOfflineCLIWorkflowAndPreview|TestImageCLIEditRoundTrip' -v ./internal/translation/cli
```

Smoke model thật **có thể tính phí**, tối đa 6 requests; chỉ kiểm tra kết nối/hợp đồng phản hồi, chưa nghiệm thu chất lượng truyện:

```powershell
$env:NOVEL_TRANSLATE_LIVE_PROVIDER = (Resolve-Path .\config\provider.jsonc).Path
go test -count=1 -run '^TestLiveConfiguredProviderSmoke$' -v ./internal/translation/model
Remove-Item Env:NOVEL_TRANSLATE_LIVE_PROVIDER
```

Race test trên môi trường có cgo/C compiler:

```sh
go test -race -buildvcs=false -count=1 ./internal/translation/state ./internal/translation/engine ./internal/translation/continuity ./internal/translation/revision ./internal/translation/model ./internal/translation/cli
```

CI khai báo test Windows/Linux, race trên Linux và một job kiểm chứng export bằng EPUBCheck thật. Workflow Docker/snapshot chạy thủ công. Có workflow không đồng nghĩa đã chạy thành công: xem logs của lần chạy cụ thể.

Kiểm chứng ngày **21/10/2026**: toàn suite offline, export với EPUBCheck thật, vet/format và build Windows/Linux/macOS amd64/arm64 đạt. Đã kiểm tra hai workspace cùng đọc file skill chung, quy ước riêng không lẫn, nội dung mặc định không đổi vẫn giữ hiệu lực bản dịch cũ, và TUI chọn/sửa thư viện chung trên Windows. Kiểm tra cấu hình Compose đạt; chưa chạy container. Các lần kiểm tra này không gọi API trả phí. Chạy TUI/installer thực tế trên Linux/macOS, race test và nghiệm thu chất lượng bằng model thật vẫn cần môi trường tương ứng.

Docker là tùy chọn: chuẩn bị `config/provider.json`, `tools/epubcheck/` và `workspace/`, rồi dùng `docker compose run --rm --build novel-translate tui`. Workspace được bind-mount nên `.env` bên trong từng workspace cũng được đọc tự động; Compose có thể chuyển `NOVEL_TRANSLATE_API_KEY` từ shell và giá trị không rỗng này được ưu tiên. Workspace trong container dùng đường dẫn Linux: đặt `provider_config: /root/.novel-translate/config.json` (hoặc bỏ trường này để dùng fallback) và checker `/tools/epubcheck/epubcheck.jar`; không dùng đường dẫn `D:/...` trong đó. Nếu thiết lập model bằng TUI, file provider và `.env` được lưu ngay trong workspace đó.

## 10. Giới hạn và xử lý lỗi

| Exit code | Ý nghĩa |
| --- | --- |
| `0` | Lệnh thành công trong phạm vi được yêu cầu |
| `1` | Lỗi cấu hình, IO hoặc request |
| `2` | Còn vấn đề chưa giải quyết; xem report/diagnostics |

- Phạm vi hiện tại là EPUB JP→VN; chưa có PDF hoặc luồng sáng tác.
- EPUB mã hóa, script, chữ SVG/CSS sinh động và cấu trúc chưa được hỗ trợ có thể chặn xuất hoàn chỉnh. Không tự bỏ qua để tuyên bố đủ nội dung.
- Trình bày giữ XHTML/spine và vị trí ảnh nguồn; không sao chép chính xác phân trang của bản mẫu hay bảo đảm giống nhau trên mọi reader.
- `keep-original` là ngoại lệ có quyết định của người dùng. Nó không chứng minh toàn bộ chữ Nhật đã được dịch.
- Chi phí chưa rõ được ghi `unknown`; một request thất bại hoặc bị hủy vẫn có thể phát sinh phí phía dịch vụ.
- Nghiệm thu model thật, đọc thử bản Việt trên reader, chạy cả tập và xuyên tập cần thực hiện trước khi dịch hàng loạt.

Lỗi thường gặp: kiểm tra `provider_config` và biến môi trường nếu báo thiếu key; dùng `resume` sau lỗi mạng; giải quyết xung đột theo `retranslate plan`; assess lại ảnh khi hash đổi; cấu hình EPUBCheck và đọc diagnostics khi final export bị chặn. Nếu Windows báo `An Application Control policy has blocked this file`, kiểm tra môi trường/chính sách thực thi và dùng CI hoặc môi trường Linux phù hợp để kiểm chứng; không đổi key để xử lý lỗi này.

## 11. Sao lưu và giấy phép

Dừng tác vụ ghi trước khi sao lưu. Sao lưu **toàn bộ workspace**, gồm YAML, skills, bản thảo, file ảnh thay thế và `.translation_work/`. Nếu ảnh hoặc provider config nằm ngoài workspace, sao lưu chúng riêng. Chỉ giữ Markdown mà bỏ snapshots/generations sẽ không đủ để resume/restore đúng trạng thái.

Module path `github.com/voocel/ainovel-cli` được giữ từ dự án gốc để bảo toàn imports/attribution; entry point của sản phẩm dịch là `cmd/novel-translate-cli`.

Phân phối theo [MIT License](LICENSE).
