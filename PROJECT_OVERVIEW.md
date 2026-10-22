# Novel Translate CLI

## Dự án này dùng để làm gì?

`novel-translate-cli` là một công cụ dòng lệnh và giao diện terminal (TUI) dùng để dịch light novel từ tiếng Nhật sang tiếng Việt.

Đầu vào chính là một hoặc nhiều file EPUB tiếng Nhật. Đầu ra là EPUB tiếng Việt, đi kèm các dữ liệu trung gian để người dùng theo dõi tiến độ, kiểm tra bản dịch, sửa thủ công, xử lý chữ trong ảnh và xuất bản hoàn chỉnh.

Công cụ được thiết kế cho một bộ truyện có nhiều tập. Mỗi tập được quản lý trong một workspace riêng; workspace lưu cấu hình, nguồn EPUB, bản dịch, lịch sử thay đổi, quy ước dịch, ngữ cảnh liên tập và các báo cáo kiểm tra.

## Vấn đề mà dự án giải quyết

Dịch một light novel dài không chỉ là gửi từng đoạn văn cho một model. Khi làm thủ công hoặc gọi model độc lập cho từng chương, các vấn đề thường gặp là:

- tên nhân vật và thuật ngữ bị dịch không nhất quán;
- cách xưng hô thay đổi giữa các chương hoặc giữa các tập;
- model thêm thông tin không có trong nguyên tác hoặc bỏ sót nội dung;
- bản dịch cũ trở nên không còn đáng tin khi nguồn, quy ước hoặc ngữ cảnh thay đổi;
- nội dung chữ trong ảnh bị bỏ qua;
- không biết chính xác phần nào đã được dịch, phần nào đã review và phần nào còn lỗi;
- file EPUB xuất ra có thể hỏng cấu trúc dù nội dung dịch đã có.

Dự án giải quyết các vấn đề này bằng cách chia bản dịch thành những khối có định danh, lưu hash của nguồn và bản dịch, lưu lịch sử generation, gắn review với đúng phiên bản dữ liệu, giữ quy ước có phạm vi áp dụng và chặn xuất bản hoàn chỉnh khi còn cổng kiểm tra chưa đạt.

## Luồng sử dụng chính

Luồng sản phẩm hiện tại là:

```text
Tạo workspace
    ↓
Đăng ký tập và file EPUB nguồn
    ↓
Import EPUB thành snapshot nguồn
    ↓
Dịch theo khối bằng model
    ↓
Review bản dịch và ngữ cảnh
    ↓
Sửa tay / giữ bản dịch có lý do / review lại
    ↓
Đánh giá chữ trong ảnh và thêm ảnh hoặc transcript đã xử lý
    ↓
Verify độ phủ, tính mới và các dependency
    ↓
Xuất EPUB preview hoặc EPUB hoàn chỉnh
    ↓
Đọc lại file xuất và chạy EPUBCheck
```

Tác vụ dịch và review có thể tiếp tục sau khi bị dừng. Kết quả hợp lệ đã lưu được dùng lại; phần chưa hoàn tất tiếp tục từ trạng thái đã commit.

## Các cách sử dụng

### Chạy bằng CLI

CLI dành cho các thao tác có thể tự động hóa hoặc chạy trong script:

- tạo và đọc workspace;
- thêm tập và đăng ký file EPUB;
- import nguồn;
- dịch, resume và review;
- đồng bộ bản sửa thủ công;
- lập và áp dụng kế hoạch khi nguồn hoặc quy ước thay đổi;
- quản lý skills, thuật ngữ và dữ liệu research;
- liệt kê, xuất và đánh giá ảnh;
- verify và export EPUB;
- xem trạng thái và usage dưới dạng text hoặc JSON.

Entry point của ứng dụng là `cmd/novel-translate-cli/main.go`.

### Chạy bằng TUI

TUI dành cho người dùng làm việc tương tác trong terminal. Nó cung cấp các màn hình để:

- mở hoặc tạo workspace;
- xem dashboard và tiến độ từng tập/chương;
- chọn phạm vi dịch hoặc review;
- theo dõi tác vụ đang chạy, pause/resume và cancel;
- đọc và sửa bản dịch;
- xem review issues, ruby và bằng chứng;
- quản lý skills, quy ước và context;
- đánh giá ảnh;
- cấu hình provider, model, giới hạn chạy và EPUBCheck;
- xem báo cáo và usage.

TUI hiện tại được khởi chạy từ `internal/translation/cli/runTUI` và dùng shell/screen ở `internal/translation/tui`.

## Đối tượng sử dụng

Dự án dành cho người dịch hoặc nhóm nhỏ muốn:

- dịch light novel Nhật → Việt theo từng tập;
- dùng model để tạo bản nháp nhưng vẫn kiểm soát việc review và sửa tay;
- duy trì thuật ngữ, tên riêng và xưng hô theo phạm vi truyện;
- giữ lại lịch sử để có thể truy vết hoặc khôi phục;
- kiểm soát chi phí và số request tới model;
- tạo EPUB có cấu trúc hợp lệ để đọc trên ebook reader.

Đây không phải là dịch vụ xuất bản tự động không cần người kiểm duyệt. Kết quả model luôn cần được review; EPUB hợp lệ về mặt kỹ thuật không đồng nghĩa với bản dịch đúng nghĩa tuyệt đối.

## Phạm vi hiện tại

### Được hỗ trợ

- nguồn EPUB;
- ngôn ngữ nguồn tiếng Nhật (`ja`);
- ngôn ngữ đích tiếng Việt (`vi`);
- workspace chứa nhiều volume/tập;
- dịch theo block và theo batch chương;
- continuity/context giữa các khối và giữa các tập có liên kết;
- quy ước dùng chung và quy ước riêng của bộ truyện;
- review bằng model với contract có cấu trúc;
- sửa tay, giữ bản dịch có lý do và re-review;
- xử lý ảnh theo quyết định của người dùng: không có chữ, giữ ảnh gốc, thay ảnh, hoặc thêm transcript;
- export preview và export final;
- readback file xuất và EPUBCheck;
- provider OpenAI-compatible, Gemini native, Anthropic native và các provider được runtime hỗ trợ;
- ghi usage, token và chi phí đã biết/chưa biết.

### Chưa phải mục tiêu của dự án

- dịch PDF hoặc các định dạng sách ngoài EPUB;
- OCR và tự động chèn chữ Việt lên ảnh;
- tự động phát hành hoặc đăng sách lên nền tảng bên ngoài;
- bảo đảm chất lượng văn chương mà không có người review;
- dùng dữ liệu tình tiết tương lai để áp vào cảnh hồi tưởng hoặc cảnh ở thời điểm trước;
- bỏ qua blocking issue để tạo bản final;
- thay thế công cụ chỉnh sửa ảnh chuyên dụng.

## Khái niệm quan trọng

### Workspace

Workspace là thư mục làm việc của một bộ truyện. Nó chứa `workspace.yaml`, danh sách volume, cấu hình output, skills riêng và dữ liệu vận hành trong `.translation_work/`.

Một workspace có thể chứa nhiều tập. Mỗi volume có ID, thứ tự, file EPUB nguồn và có thể liên kết với tập trước để lấy context đã được chấp nhận.

### Source snapshot

File EPUB bên ngoài workspace được coi là nguồn đầu vào và chỉ được đọc. Khi import, ứng dụng lưu một bản snapshot cùng manifest, hash và mapping các document/chapter/block. Nếu file nguồn thay đổi, mapping cũ không được âm thầm dùng tiếp; người dùng phải lập và áp dụng source revision plan.

### Block

Block là đơn vị nội dung nhỏ nhất được theo dõi. Block có ID, vị trí, source text, source hash, loại nội dung, locator EPUB và disposition. Block có thể là text, attribute, ảnh hoặc nội dung khác mà parser nhận diện được.

### Unit và generation

Unit là bản dịch của một block. Unit lưu target, revision, nguồn tạo ra bản dịch, review, dependencies, context events và trạng thái stale.

Generation là một phiên bản bất biến của toàn bộ trạng thái dịch của một volume. Generation có parent, source hash, ID, timestamp và được lưu cùng checksum. Con trỏ `current` chỉ tới generation đang được dùng.

### Accepted

Một unit chỉ được coi là accepted khi:

- không có stale reason;
- có review;
- review khớp source hash hiện tại;
- review khớp target hash hiện tại;
- quyết định là `accepted` hoặc `user-kept`.

Verify còn kiểm tra source, skills, rules và context dependency. Vì vậy một unit từng accepted có thể phải review lại khi các dependency liên quan thay đổi.

### Skills và rules

Skills là hướng dẫn dịch dạng Markdown. Có skill dùng chung cho nhiều workspace và skill riêng cho từng series. Rules là các quy ước có cấu trúc, có phạm vi theo vị trí truyện, bằng chứng và trạng thái như candidate, confirmed, rejected hoặc conflict.

Quy ước riêng của series có độ ưu tiên cao hơn skill dùng chung. Quy tắc chưa có đủ bằng chứng không được tự động coi là sự thật; candidate hoặc conflict phải được người dùng quyết định.

### Review và signals

Review model trả về issues, ruby decisions, decision và bằng chứng source/target. Signals trong code chỉ là dấu hiệu hỗ trợ reviewer, không phải điểm số chất lượng ngữ nghĩa tự động.

### Image decision

Mỗi occurrence của ảnh cần một quyết định rõ ràng trước khi export final. `keep-original` chỉ có nghĩa người dùng chủ động giữ ảnh gốc; nó không chứng minh chữ trong ảnh đã được dịch.

## Dữ liệu người dùng nhìn thấy

Một workspace sau khi làm việc thường có dạng:

```text
workspace/<series>/
├── workspace.yaml                 # cấu hình series, volumes, run, output
├── .env                           # API key cục bộ nếu người dùng lưu tại đây
├── Skills/
│   ├── SKILL.md                   # quy ước riêng của series
│   ├── rules.json                 # rules đã import/generate/resolve
│   └── imported/                  # references đã import
├── <volume-directory>/
│   ├── chapters/                  # Markdown view có unit markers
│   └── images/                    # ảnh nguồn hoặc ảnh do người dùng xử lý
├── Output/                        # EPUB và report sau export
└── .translation_work/             # snapshot, generation, job, usage, evidence, plan
```

Các file trong `.translation_work/` là một phần của trạng thái sản phẩm. Chỉ giữ Markdown hoặc chỉ giữ EPUB output là không đủ để resume, verify, restore hay truy vết chính xác.

## Các cổng an toàn của sản phẩm

- Chỉ ghi dữ liệu bền vững trong workspace hoặc thư mục được cấu hình hợp lệ.
- Mỗi workspace có writer lock để tránh hai tiến trình cùng commit.
- Ghi file dùng thao tác atomic; generation có checksum.
- Commit generation dùng expected parent/current ID để phát hiện dữ liệu đã thay đổi.
- Không dùng source snapshot stale khi file nguồn hoặc parser manifest đã đổi.
- Không coi bản dịch là hoàn tất nếu còn block chưa dịch, review stale, blocking issue, xung đột rule hoặc image decision chưa xử lý.
- Không tự động overwrite bản sửa thủ công khi mapping hoặc marker không còn chắc chắn.
- Request model được giới hạn theo context, timeout, retry, max requests và budget.
- Usage không có giá được ghi là unknown, không giả định là miễn phí.

## Cấu trúc mã nguồn ở mức khái niệm

```text
cmd/novel-translate-cli
        ↓
internal/translation/cli
        ├── CLI command parser và một số orchestration trực tiếp
        └── launcher của TUI hiện tại
                ↓
internal/translation/app
        ├── use case và query cho TUI
        ├── JobManager cho tác vụ nền
        └── import / review / edit / verify / export
                ↓
internal/translation/engine, epub, revision, skills, verify, model, state
```

Các package nền nằm ngoài `translation` gồm:

- `internal/bootstrap`: đọc provider config, tạo model, failover và context limits;
- `internal/models`: registry và pricing metadata của model;
- `internal/llmcontract`: schema, structured output và validation contract;
- `internal/llmretry`: retry transport có giới hạn;
- `internal/errs`: error helpers dùng chung.

Các package nghiệp vụ chính gồm:

- `domain`: các DTO được lưu trong generation và manifest;
- `workspace`: đọc, validate và ghi `workspace.yaml`;
- `state`: atomic write, lock, pointer, checksum và generation commit;
- `epub`: đọc ZIP/XML EPUB, lập inventory, scope và ghi EPUB;
- `engine`: source snapshot, batch, translation/review runner, context và source revision;
- `model`: adapter gọi provider, contract execution và usage journal;
- `skills`: load skill, scoped rules, shared library, import và generate candidates;
- `continuity`: entities, context events và story-time boundaries;
- `revision`: Markdown views, sync, manual edit và impact plan;
- `verify`: image assessment, signals và EPUBCheck;
- `tui`: shell, screen, component và theme của giao diện hiện tại;
- `app`: lớp use case/projection mà TUI hiện tại dùng.

## Cấu hình model

Provider được khai báo trong JSON/JSONC. Có thể dùng cấu hình mặc định hoặc trỏ `provider_config` từ `workspace.yaml`. Ba role chính là:

- `translation`: tạo bản dịch;
- `translation_review`: review bản dịch;
- `translation_research`: đề xuất quy ước có bằng chứng.

API key được lấy từ biến môi trường hoặc `.env` cục bộ theo `api_key_env`. Không commit key vào repo, skill hoặc workspace config được theo dõi trong Git.

Model adapter thực hiện các việc sau trước và sau request:

1. tính input limit theo context window và run settings;
2. áp dụng timeout, max output tokens, retry và contract correction;
3. ghi request journal trước khi gọi model;
4. ghi usage, model thực tế, cost hoặc unknown sau request;
5. dừng khi chạm max requests hoặc budget không còn an toàn.

## Cách kiểm tra dự án

Các lệnh kiểm tra cơ bản:

```powershell
go vet ./...
go test -buildvcs=false -count=1 ./...
git diff --check
```

Build và chạy từ source:

```powershell
go build -trimpath -ldflags="-s -w" -o bin/novel-translate-cli.exe ./cmd/novel-translate-cli
.\scripts\run.ps1
```

`scripts/run.ps1` và `scripts/run.sh` mặc định mở TUI; tham số truyền sau script được chuyển cho CLI. `make test`, `make vet` và `make build` là các shortcut tương ứng trên môi trường có Make.

## Những điều tài liệu này dùng làm nền cho refactor

Tài liệu này mô tả **hành vi và ranh giới sản phẩm hiện tại**, không khẳng định cấu trúc code hiện tại đã sạch. Khi lập kế hoạch refactor cần giữ các hành vi sau:

- CLI và TUI phải dùng chung các cổng nghiệp vụ và validation;
- state, source snapshot, generation và revision phải giữ tính truy vết;
- không làm mất khả năng resume, restore, sync, verify và export;
- không làm mất dữ liệu workspace hiện có;
- thay đổi package phải giữ được test offline và model caller injectable;
- mọi thay đổi ảnh hưởng schema hoặc file layout phải có migration hoặc tương thích đọc dữ liệu cũ;
- các phần code cũ chỉ được loại bỏ sau khi chứng minh không còn runtime caller hoặc đã có đường thay thế được test.

Khi code và tài liệu mâu thuẫn, ưu tiên kiểm tra code đang được gọi từ `cmd/novel-translate-cli`, các test hành vi và dữ liệu workspace thực tế; sau đó cập nhật lại tài liệu này.
