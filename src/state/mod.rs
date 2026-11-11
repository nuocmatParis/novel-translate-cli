pub mod context;
pub mod export;
pub mod logs;
pub mod review;
pub mod skills;
pub mod workflow;
pub mod workspace;

use context::*;
use export::*;
use logs::*;
use review::*;
use skills::*;
use workflow::*;
use workspace::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoScenario {
    NormalTranslation,
    ReviewIssuesDetected,
    WaitingForTerminologyApproval,
    ModelRequestFailed,
    BudgetLimitReached,
    TranslationCompleted,
    ExportValidationFailed,
}

impl DemoScenario {
    pub fn all() -> &'static [DemoScenario] {
        &[
            DemoScenario::NormalTranslation,
            DemoScenario::ReviewIssuesDetected,
            DemoScenario::WaitingForTerminologyApproval,
            DemoScenario::ModelRequestFailed,
            DemoScenario::BudgetLimitReached,
            DemoScenario::TranslationCompleted,
            DemoScenario::ExportValidationFailed,
        ]
    }

    pub fn key(&self) -> &'static str {
        match self {
            DemoScenario::NormalTranslation => "F1",
            DemoScenario::ReviewIssuesDetected => "F2",
            DemoScenario::WaitingForTerminologyApproval => "F3",
            DemoScenario::ModelRequestFailed => "F4",
            DemoScenario::BudgetLimitReached => "F5",
            DemoScenario::TranslationCompleted => "F6",
            DemoScenario::ExportValidationFailed => "F7",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            DemoScenario::NormalTranslation => "1. Normal Translation",
            DemoScenario::ReviewIssuesDetected => "2. Review Issues Detected",
            DemoScenario::WaitingForTerminologyApproval => "3. Waiting for Terminology Approval",
            DemoScenario::ModelRequestFailed => "4. Model Request Failed",
            DemoScenario::BudgetLimitReached => "5. Budget Limit Reached",
            DemoScenario::TranslationCompleted => "6. Translation Completed",
            DemoScenario::ExportValidationFailed => "7. Export Validation Failed",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DemoScenario::NormalTranslation => "Normal translating batch; smooth live workflow",
            DemoScenario::ReviewIssuesDetected => {
                "3 issues detected in review (S1, S2, S3), action required"
            }
            DemoScenario::WaitingForTerminologyApproval => {
                "New unapproved terms pending human review in Skills"
            }
            DemoScenario::ModelRequestFailed => {
                "Upstream 503 error / rate-limit encountered with retry option"
            }
            DemoScenario::BudgetLimitReached => {
                "Cost cap reached ($5.00); workflow halted until cap raised"
            }
            DemoScenario::TranslationCompleted => {
                "All 6 chapters completed; ready for final staging & export"
            }
            DemoScenario::ExportValidationFailed => {
                "Review issues unresolved & EPUBCheck error blocks final export"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub workspace: MockWorkspace,
    pub task: MockTranslationTask,
    pub review_issues: Vec<MockReviewIssue>,
    pub editor_blocks: Vec<MockEditorBlock>,
    pub characters: Vec<MockCharacter>,
    pub terminology: Vec<MockTerminology>,
    pub addressing: Vec<MockAddressing>,
    pub relationships: Vec<MockRelationship>,
    pub timeline: Vec<MockTimelineEvent>,
    pub skills: Vec<MockSkillDoc>,
    pub decisions: Vec<MockPendingDecision>,
    pub export: MockExportStatus,
    pub logs: Vec<MockLogEntry>,
    pub active_scenario: DemoScenario,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut state = Self::build_base_state();
        state.apply_scenario(DemoScenario::NormalTranslation);
        state
    }

    fn build_base_state() -> Self {
        let chapters_v26 = vec![
            MockChapter {
                id: "c00".to_string(),
                index: 0,
                title: "Thức tỉnh nơi phương xa".to_string(),
                original_title: "異郷の目覚め".to_string(),
                status: ChapterStatus::Completed,
                total_blocks: 120,
                translated_blocks: 120,
                reviewed_blocks: 120,
                selected: false,
            },
            MockChapter {
                id: "c01".to_string(),
                index: 1,
                title: "Kẻ mất trí và cô bé bán quỷ".to_string(),
                original_title: "記憶喪失者と鬼族の少女".to_string(),
                status: ChapterStatus::Completed,
                total_blocks: 154,
                translated_blocks: 154,
                reviewed_blocks: 154,
                selected: false,
            },
            MockChapter {
                id: "c02".to_string(),
                index: 2,
                title: "Khu rừng của tộc Shudraq".to_string(),
                original_title: "シュドラクの民の森".to_string(),
                status: ChapterStatus::Completed,
                total_blocks: 180,
                translated_blocks: 180,
                reviewed_blocks: 180,
                selected: false,
            },
            MockChapter {
                id: "c03".to_string(),
                index: 3,
                title: "Lời phán quyết của Tộc Trưởng".to_string(),
                original_title: "族長の審判".to_string(),
                status: ChapterStatus::Completed,
                total_blocks: 165,
                translated_blocks: 165,
                reviewed_blocks: 165,
                selected: false,
            },
            MockChapter {
                id: "c04".to_string(),
                index: 4,
                title: "Huyết chiến săn mồi".to_string(),
                original_title: "血闘の狩猟".to_string(),
                status: ChapterStatus::Translating,
                total_blocks: 195,
                translated_blocks: 124,
                reviewed_blocks: 98,
                selected: true,
            },
            MockChapter {
                id: "c05".to_string(),
                index: 5,
                title: "Hoàng đế phế truất".to_string(),
                original_title: "廃帝の策略".to_string(),
                status: ChapterStatus::Queued,
                total_blocks: 172,
                translated_blocks: 0,
                reviewed_blocks: 0,
                selected: true,
            },
            MockChapter {
                id: "c06".to_string(),
                index: 6,
                title: "Ngọn cờ khởi nghĩa".to_string(),
                original_title: "叛乱の烽火".to_string(),
                status: ChapterStatus::Queued,
                total_blocks: 140,
                translated_blocks: 0,
                reviewed_blocks: 0,
                selected: false,
            },
        ];

        let volumes = vec![
            MockVolume {
                id: "v01".to_string(),
                number: 1,
                title: "Tập 01 — Một ngày hỗn loạn ở Vương Đô".to_string(),
                status: VolumeStatus::Completed,
                progress_percent: 100,
                chapters: vec![],
            },
            MockVolume {
                id: "v02".to_string(),
                number: 2,
                title: "Tập 02 — Tuần lễ đầy trắc trở tại Dinh Thự Roswaal".to_string(),
                status: VolumeStatus::Completed,
                progress_percent: 100,
                chapters: vec![],
            },
            MockVolume {
                id: "v26".to_string(),
                number: 26,
                title: "Tập 26 — Đế quốc Sói Lạc Lối Vollachia (Arc 7)".to_string(),
                status: VolumeStatus::InProgress,
                progress_percent: 64,
                chapters: chapters_v26,
            },
            MockVolume {
                id: "v27".to_string(),
                number: 27,
                title: "Tập 27 — Thành quách hỗn loạn".to_string(),
                status: VolumeStatus::NotStarted,
                progress_percent: 0,
                chapters: vec![],
            },
        ];

        let series = MockSeries {
            title: "Re:Zero − Bắt đầu lại ở thế giới khác".to_string(),
            original_title: "Re:ゼロから始める異世界生活".to_string(),
            author: "Tappei Nagatsuki".to_string(),
            illustrator: "Shinichirou Otsuka".to_string(),
            description: "Cậu học sinh trung học Natsuki Subaru đột nhiên bị triệu hồi sang một dị giới kỳ ảo...".to_string(),
            volumes,
        };

        let workspace = MockWorkspace {
            name: "rezero-arc7-workspace".to_string(),
            root_path: "D:/FPTU/novel-translate-cli/workspace/rezero".to_string(),
            series,
            active_volume_index: 2,  // Volume 26
            active_chapter_index: 4, // Chapter 04
        };

        let task = MockTranslationTask {
            is_running: false,
            is_paused: false,
            mode: TranslationMode::SelectedChapters,
            current_stage: WorkflowStage::TranslatingBatch,
            stage_states: [
                StepState::Completed,
                StepState::Running,
                StepState::Pending,
                StepState::Pending,
                StepState::Pending,
                StepState::Pending,
            ],
            current_batch: 3,
            total_batches: 5,
            volume_progress_pct: 64,
            chapter_progress_pct: 63,
            active_model: MockModelConfig {
                provider: "OpenAI".to_string(),
                model_name: "gpt-4o".to_string(),
                role: ModelRole::Translation,
                temperature: 0.3,
                max_tokens: 4096,
            },
            current_operation: "Translating Japanese block batch #B041-#B050 with glossary context"
                .to_string(),
            session_tokens: 142_500,
            session_cost_usd: 0.71,
            budget_limit_usd: Some(5.00),
            budget_unlimited: false,
        };

        let review_issues = vec![
            MockReviewIssue {
                id: "ISSUE-001".to_string(),
                volume_num: 26,
                chapter_title: "Huyết chiến săn mồi".to_string(),
                block_id: "#B042".to_string(),
                category: "Terminology / Thuật ngữ".to_string(),
                severity: Severity::S2Major,
                action_scope: ActionScope::M1DirectFix,
                evidence_status: EvidenceStatus::Confirmed,
                japanese_source: "「お前は本当に『死に戻り』の意味を理解しているのか？」".to_string(),
                vietnamese_translation: "「Ngươi có thực sự hiểu ý nghĩa của việc 'chết đi sống lại' không?」".to_string(),
                explanation: "Sử dụng sai thuật ngữ quy ước cốt lõi của series. '死に戻り' chuẩn là 'Trở về bằng cái chết', không dùng 'chết đi sống lại' chung chung.".to_string(),
                suggested_correction: "「Ngươi có thực sự hiểu ý nghĩa của 'Trở về bằng cái chết' không?」".to_string(),
                cycles_count: 2,
                max_cycles: 5,
                resolved: false,
                user_exception: false,
                feedback_notes: vec![
                    "Lần 1: Reviewer gắn cờ thuật ngữ quy ước chưa nhất quán".to_string(),
                    "Lần 2: Translation model giữ nguyên 'chết đi sống lại' do thiếu context tag".to_string(),
                ],
            },
            MockReviewIssue {
                id: "ISSUE-002".to_string(),
                volume_num: 26,
                chapter_title: "Huyết chiến săn mồi".to_string(),
                block_id: "#B057".to_string(),
                category: "Addressing / Xưng hô".to_string(),
                severity: Severity::S1Minor,
                action_scope: ActionScope::M1DirectFix,
                evidence_status: EvidenceStatus::Confirmed,
                japanese_source: "「スバルくん、レムは信じています」".to_string(),
                vietnamese_translation: "「Anh Subaru, em tin anh」".to_string(),
                explanation: "Quy ước xưng hô nhân vật: Rem gọi Subaru là 'Subaru-kun' giữ hậu tố, xưng tên ngôi thứ ba 'Rem' chứ không xưng 'em'.".to_string(),
                suggested_correction: "「Subaru-kun, Rem luôn tin tưởng cậu」".to_string(),
                cycles_count: 1,
                max_cycles: 5,
                resolved: false,
                user_exception: false,
                feedback_notes: vec![
                    "Lần 1: Khớp với quy tắc xưng hô Arc 7".to_string(),
                ],
            },
            MockReviewIssue {
                id: "ISSUE-003".to_string(),
                volume_num: 26,
                chapter_title: "Huyết chiến săn mồi".to_string(),
                block_id: "#B081".to_string(),
                category: "Meaning / Lệch sự kiện".to_string(),
                severity: Severity::S3Critical,
                action_scope: ActionScope::M2RetranslateBlock,
                evidence_status: EvidenceStatus::Confirmed,
                japanese_source: "「ヴィンセントは剣を抜かず、ただ冷酷な笑みを浮かべたまま告げた」".to_string(),
                vietnamese_translation: "「Vincent rút kiếm ra và mỉm cười ấm áp bảo」".to_string(),
                explanation: "Lệch hoàn toàn hành động nhân vật: '剣を抜かず' (không rút kiếm) bị dịch thành 'rút kiếm ra'; '冷酷な笑み' (nụ cười lạnh lùng) bị dịch thành 'mỉm cười ấm áp'.".to_string(),
                suggested_correction: "「Vincent không hề rút kiếm, chỉ giữ nụ cười lạnh lùng và cất lời」".to_string(),
                cycles_count: 5,
                max_cycles: 5,
                resolved: false,
                user_exception: false,
                feedback_notes: vec![
                    "Vòng 1-4: Model liên tục hallucinate sắc thái nhân vật".to_string(),
                    "Vòng 5: Đã chạm ngưỡng 5 lần sửa/review. Cần người dùng can thiệp!".to_string(),
                ],
            },
        ];

        let editor_blocks = vec![
            MockEditorBlock {
                id: "B041".to_string(),
                block_num: 41,
                japanese: "暗がりの中、かすかな息遣いとともに男の影がゆっくりと立ち上がった。".to_string(),
                vietnamese: "Trong bóng tối mịt mờ, bóng đen của người đàn ông từ từ gượng dậy cùng hơi thở yếu ớt dồn dập.".to_string(),
                revision: 1,
                review_status: "Approved".to_string(),
                last_modified: "10:12:05".to_string(),
                history: vec!["Rev 1: Dịch ban đầu bởi Claude 3.5 Sonnet".to_string()],
            },
            MockEditorBlock {
                id: "B042".to_string(),
                block_num: 42,
                japanese: "「お前は本当に『死に戻り』の意味を理解しているのか？」ヴィンセントは鋭い眼光で問いただした。".to_string(),
                vietnamese: "「Ngươi có thực sự hiểu ý nghĩa của việc 'chết đi sống lại' không?」Vincent chất vấn với ánh mắt sắc như dao cạo.".to_string(),
                revision: 2,
                review_status: "Pending Fix (Issue #001)".to_string(),
                last_modified: "10:14:22".to_string(),
                history: vec![
                    "Rev 1: Dịch ban đầu bởi Claude 3.5 Sonnet".to_string(),
                    "Rev 2: Cập nhật ngữ điệu theo gợi ý Reviewer".to_string(),
                ],
            },
            MockEditorBlock {
                id: "B043".to_string(),
                block_num: 43,
                japanese: "スバルは喉の渇きを覚えながら、震える拳を強く握りしめた。「……何度も、嫌になるほど味わってきたさ」".to_string(),
                vietnamese: "Subaru cảm thấy cổ họng khô khốc, nắm chặt nắm đấm đang run rẩy. \"……Đã trải qua vô số lần, đến mức muốn phát nôn rồi.\"".to_string(),
                revision: 1,
                review_status: "Approved".to_string(),
                last_modified: "10:12:10".to_string(),
                history: vec!["Rev 1: Dịch ban đầu bởi Claude 3.5 Sonnet".to_string()],
            },
            MockEditorBlock {
                id: "B044".to_string(),
                block_num: 44,
                japanese: "風が森の木々を揺らし、遠くで魔獣の咆哮が木霊した。夜はまだ始まったばかりだ。".to_string(),
                vietnamese: "Cơn gió làm rung chuyển cây rừng, tiếng gầm của ma thú từ đằng xa vang vọng. Đêm nay mới chỉ là điểm khởi đầu.".to_string(),
                revision: 1,
                review_status: "Approved".to_string(),
                last_modified: "10:12:12".to_string(),
                history: vec!["Rev 1: Dịch ban đầu bởi Claude 3.5 Sonnet".to_string()],
            },
            MockEditorBlock {
                id: "B045".to_string(),
                block_num: 45,
                japanese: "「ならいい。貴様の覚悟、見せてもらうぞ、ナツキ・スバル」".to_string(),
                vietnamese: "「Vậy thì tốt. Hãy cho ta thấy sự giác ngộ của ngươi, Natsuki Subaru.」".to_string(),
                revision: 1,
                review_status: "Approved".to_string(),
                last_modified: "10:12:15".to_string(),
                history: vec!["Rev 1: Dịch ban đầu bởi Claude 3.5 Sonnet".to_string()],
            },
        ];

        let characters = vec![
            MockCharacter {
                name: "Natsuki Subaru".to_string(),
                original_name: "ナツキ・スバル (菜月 昴)".to_string(),
                alias: "Lolimancer / Hiệp sĩ của Emilia".to_string(),
                gender: "Nam".to_string(),
                role: "Nhân vật chính".to_string(),
                description: "Sở hữu năng lực 'Trở về bằng cái chết' sau khi bị triệu hồi sang thế giới khác.".to_string(),
            },
            MockCharacter {
                name: "Emilia".to_string(),
                original_name: "エミリア".to_string(),
                alias: "Lia / Phù Thủy Băng Tuyết".to_string(),
                gender: "Nữ".to_string(),
                role: "Nữ chính / Ứng viên Quốc Vương Lugnica".to_string(),
                description: "Bán elf tóc bạc mắt tím, tính cách ngây thơ, tốt bụng và mang gánh nặng huyết thống phù thủy.".to_string(),
            },
            MockCharacter {
                name: "Rem".to_string(),
                original_name: "レム".to_string(),
                alias: "Hầu gái dinh thự Roswaal / Quỷ tộc".to_string(),
                gender: "Nữ".to_string(),
                role: "Nữ phụ quan trọng".to_string(),
                description: "Hầu gái quỷ tộc tóc lam, em gái của Ram. Trong Arc 7 tạm thời mất ký ức nhưng đồng hành cùng Subaru.".to_string(),
            },
            MockCharacter {
                name: "Ram".to_string(),
                original_name: "ラム".to_string(),
                alias: "Hầu gái trưởng / Quỷ thần gãy sừng".to_string(),
                gender: "Nữ".to_string(),
                role: "Hầu gái dinh thự Roswaal".to_string(),
                description: "Chị gái sinh đôi của Rem, tóc hồng, miệng lưỡi sắc bén, gọi Subaru là 'Barusu'.".to_string(),
            },
            MockCharacter {
                name: "Vincent Vollachia".to_string(),
                original_name: "ヴィンセント・ヴォラキア".to_string(),
                alias: "Abel / Hoàng đế thứ 77 của Vollachia".to_string(),
                gender: "Nam".to_string(),
                role: "Đồng minh bất đắc dĩ / Arc 7".to_string(),
                description: "Hoàng đế đế quốc Vollachia bị lật đổ, đeo mặt nạ quỷ và cùng Subaru giành lại ngai vàng.".to_string(),
            },
        ];

        let terminology = vec![
            MockTerminology {
                source_term: "死に戻り (Shinimodori)".to_string(),
                vietnamese_translation: "Trở về bằng cái chết".to_string(),
                notes: "Quyền năng cốt lõi của Subaru. Bắt buộc thống nhất toàn bộ series."
                    .to_string(),
                scope: "Series Scope (Global)".to_string(),
                source_file: "rezero_quy_uoc_dich_va_xung_ho.md".to_string(),
            },
            MockTerminology {
                source_term: "権能 (Kennō)".to_string(),
                vietnamese_translation: "Quyền Năng".to_string(),
                notes:
                    "Sức mạnh ma thuật đặc biệt của Đại Tội Ty Giáo và Phù Thủy, viết hoa cả 2 từ."
                        .to_string(),
                scope: "Series Scope (Global)".to_string(),
                source_file: "rezero_quy_uoc_dich_va_xung_ho.md".to_string(),
            },
            MockTerminology {
                source_term: "魔女因子 (Majo Inshi)".to_string(),
                vietnamese_translation: "Nhân Tố Phù Thủy".to_string(),
                notes: "Vật chất tà ác cội nguồn của Quyền Năng, cư ngụ trong cơ thể vật chủ."
                    .to_string(),
                scope: "Series Scope (Global)".to_string(),
                source_file: "rezero_quy_uoc_dich_va_xung_ho.md".to_string(),
            },
            MockTerminology {
                source_term: "加護 (Kago)".to_string(),
                vietnamese_translation: "Gia Hộ".to_string(),
                notes: "Phước lành của thế giới ban tặng cho các cá nhân từ lúc chào đời."
                    .to_string(),
                scope: "Series Scope (Global)".to_string(),
                source_file: "rezero_quy_uoc_dich_va_xung_ho.md".to_string(),
            },
            MockTerminology {
                source_term: "オド・ラグナ (Odo Raguna)".to_string(),
                vietnamese_translation: "Cội Nguồn Linh Hồn (Od Laguna)".to_string(),
                notes: "Tâm điểm luân hồi của vạn vật và ma lực trong thế giới Re:Zero."
                    .to_string(),
                scope: "Series Scope (Global)".to_string(),
                source_file: "Library_COMMON_KANJI_SINO_VN.md".to_string(),
            },
        ];

        let addressing = vec![
            MockAddressing {
                speaker: "Subaru".to_string(),
                listener: "Rem".to_string(),
                self_reference: "tôi / Subaru (ore)".to_string(),
                listener_reference: "Rem (tên gọi thân mật)".to_string(),
                scope: "Toàn bộ Series".to_string(),
                context: "Thân thiết, chân thành, che chở".to_string(),
                notes: "Subaru coi Rem là chỗ dựa tinh thần không thể thay thế.".to_string(),
            },
            MockAddressing {
                speaker: "Rem".to_string(),
                listener: "Subaru".to_string(),
                self_reference: "Rem (ngôi thứ 3)".to_string(),
                listener_reference: "Subaru-kun (スバルくん)".to_string(),
                scope: "Toàn bộ Series (Trước khi mất ký ức)".to_string(),
                context: "Tôn kính, sùng bái và yêu thương sâu sắc".to_string(),
                notes: "Giữ nguyên hậu tố -kun theo văn phong LN bản dịch tiếng Việt.".to_string(),
            },
            MockAddressing {
                speaker: "Rem (Arc 7 - Mất trí nhớ)".to_string(),
                listener: "Subaru".to_string(),
                self_reference: "tôi (watashi)".to_string(),
                listener_reference: "ngươi / anh ta (anata / omae)".to_string(),
                scope: "Chỉ áp dụng Tập 26-27 (Arc 7 Vollachia)".to_string(),
                context: "Nghi ngờ, sợ mùi phù thủy nồng nặc của Subaru".to_string(),
                notes: "ĐẶC BIỆT: Quy ước xưng hô Vol 26-27 không dùng làm mặc định toàn series!"
                    .to_string(),
            },
            MockAddressing {
                speaker: "Ram".to_string(),
                listener: "Subaru".to_string(),
                self_reference: "Ram / ta".to_string(),
                listener_reference: "Barusu (バルス) / tên ngốc".to_string(),
                scope: "Toàn bộ Series".to_string(),
                context: "Trêu chọc, mỉa mai nhưng ngầm công nhận".to_string(),
                notes: "Tuyệt đối không dịch Barusu thành từ khác, đây là biệt danh độc quyền."
                    .to_string(),
            },
            MockAddressing {
                speaker: "Vincent".to_string(),
                listener: "Subaru".to_string(),
                self_reference: "trẫm / ta (yo)".to_string(),
                listener_reference: "ngươi / Natsuki Subaru (kisama)".to_string(),
                scope: "Arc 7 (Tập 26-33)".to_string(),
                context: "Uy quyền đế vương, thực dụng và sắc bén".to_string(),
                notes: "Vincent luôn nói giọng thượng vị của hoàng đế Vollachia.".to_string(),
            },
        ];

        let relationships = vec![
            MockRelationship {
                from_char: "Subaru".to_string(),
                to_char: "Vincent".to_string(),
                relation_type: "Đồng minh sinh tử (Bất đắc dĩ)".to_string(),
                notes: "Cùng sống sót trong rừng Shudraq và mưu đồ phản công giành lại đế quốc.".to_string(),
            },
            MockRelationship {
                from_char: "Rem".to_string(),
                to_char: "Subaru".to_string(),
                relation_type: "Người hùng / Đang dần hồi phục niềm tin".to_string(),
                notes: "Dù mất trí nhớ ở đầu Vol 26, Rem vẫn nhận thấy sự quyết tử bảo vệ mình của Subaru.".to_string(),
            },
        ];

        let timeline = vec![
            MockTimelineEvent {
                chapter: "Prologue".to_string(),
                point: "Bị dịch chuyển đến Vollachia".to_string(),
                summary: "Subaru và Rem bị làn sương bí ẩn dịch chuyển khỏi tháp Pleiades đến rừng sâu Vollachia.".to_string(),
            },
            MockTimelineEvent {
                chapter: "Chapter 01-03".to_string(),
                point: "Gặp gỡ tộc Shudraq và Abel".to_string(),
                summary: "Vượt qua thử thách sinh tồn 'Nghi thức tắm máu' và kết liên minh với hoàng đế Abel.".to_string(),
            },
            MockTimelineEvent {
                chapter: "Chapter 04".to_string(),
                point: "Huyết chiến săn ma thú rừng rậm".to_string(),
                summary: "Tiến công ổ ma thú để chiếm lĩnh vị trí chiến lược cho kế hoạch đoạt thành Guaral.".to_string(),
            },
        ];

        let skills = vec![
            MockSkillDoc {
                filename: "Library_COMMON_KANJI_SINO_VN.md".to_string(),
                title: "Quy ước chuyển tự Hán Việt và thuật ngữ chung".to_string(),
                category: "Shared Skills (Dùng chung)".to_string(),
                scope: "Workspace Global".to_string(),
                content: "# Quy ước chuyển tự Hán-Việt & Thuật ngữ Chung\n\n1. Danh từ riêng và địa danh phương Tây: giữ nguyên phiên âm Latinh tiêu chuẩn.\n2. Thuật ngữ Phật giáo / Đạo giáo: ưu tiên phiên âm Hán-Việt trang trọng.\n3. Các tiền tố danh xưng: Hiệp sĩ, Thánh Kiếm, Thần Linh, Ma Tộc.\n4. Đơn vị đo lường: quy đổi tương đương hoặc giữ nguyên theo quy ước tác phẩm.".to_string(),
            },
            MockSkillDoc {
                filename: "rezero_quy_uoc_dich_va_xung_ho.md".to_string(),
                title: "Re:Zero — Quy ước dịch thuật, thuật ngữ và xưng hô đặc thù".to_string(),
                category: "Series Skills (Bộ truyện)".to_string(),
                scope: "Re:Zero kara Hajimeru Isekai Seikatsu".to_string(),
                content: "# Quy ước dịch Re:Zero (Nagatsuki Tappei)\n\n## 1. Thuật ngữ cốt lõi\n- 死に戻り (Shinimodori) -> Trở về bằng cái chết (KHÔNG dùng 'chết đi sống lại')\n- 権能 (Kennō) -> Quyền Năng (viết hoa cả hai chữ)\n- 魔女因子 (Majo Inshi) -> Nhân Tố Phù Thủy\n- 加護 (Kago) -> Gia Hộ\n\n## 2. Xưng hô nhân vật đặc thù\n- Rem gọi Subaru là 'Subaru-kun' và xưng tên 'Rem'\n- Ram gọi Subaru là 'Barusu'\n- Beatrice gọi Subaru là 'Subaru' và kết câu bằng 'nanoyo' ('đấy nhé' / 'đấy mà')\n\n## 3. Lưu ý Arc 7 (Vol 26-27)\n- Rem mất trí nhớ xưng 'watashi' và gọi Subaru là 'ngươi/anh ta'. Đây là trạng thái tạm thời, KHÔNG ghi đè quy tắc toàn series!".to_string(),
            },
        ];

        let decisions = vec![
            MockPendingDecision {
                id: "DEC-001".to_string(),
                original_term: "大罪司教 (Taizai Shikyō)".to_string(),
                suggestions: vec![
                    "Đại Tội Ty Giáo (Hán Việt chuẩn LN)".to_string(),
                    "Tổng Giám Mục Đại Tội (Thuật ngữ Kitô giáo)".to_string(),
                    "Giáo Chủ Đại Tội".to_string(),
                ],
                selected_index: 0,
                custom_input: "".to_string(),
                reasoning: "Ty Giáo phản ánh đúng tổ chức dị giáo huyền bí trong bối cảnh văn phong light novel, tránh hiểu lầm với phẩm trật Công giáo La Mã.".to_string(),
                scope: "Series Scope (Toàn bộ series)".to_string(),
                is_confirmed: false,
                is_deferred: false,
            },
            MockPendingDecision {
                id: "DEC-002".to_string(),
                original_term: "星名 (Hoshina / Star Names)".to_string(),
                suggestions: vec![
                    "Tên các chòm sao (Giữ nguyên phiên âm thiên văn)".to_string(),
                    "Tinh danh".to_string(),
                ],
                selected_index: 0,
                custom_input: "".to_string(),
                reasoning: "Tác giả Tappei Nagatsuki đặt tên các Giám mục và nhân vật phản diện theo tên các ngôi sao thiên văn thực tế (Petelgeuse, Regulus, Capella...).".to_string(),
                scope: "Series Scope (Toàn bộ series)".to_string(),
                is_confirmed: false,
                is_deferred: false,
            },
        ];

        let export = MockExportStatus {
            is_validating: false,
            checklist: vec![
                ValidationItem {
                    name: "Translation completed".to_string(),
                    passed: true,
                    details: "Tất cả các block trong phạm vi chọn đã được dịch".to_string(),
                },
                ValidationItem {
                    name: "Review completed".to_string(),
                    passed: true,
                    details: "Tất cả các cảnh báo S1-S3 đã được xác nhận hoặc giải quyết"
                        .to_string(),
                },
                ValidationItem {
                    name: "Structure valid".to_string(),
                    passed: true,
                    details: "Thẻ XML, cú pháp XHTML và mốc ngắt đoạn nguyên vẹn".to_string(),
                },
                ValidationItem {
                    name: "Resource mapping valid".to_string(),
                    passed: true,
                    details: "Đường dẫn 18 ảnh minh họa khớp 100% với manifest".to_string(),
                },
                ValidationItem {
                    name: "EPUBCheck passed".to_string(),
                    passed: true,
                    details: "EPUB 3.2 schema validation: 0 errors, 0 warnings".to_string(),
                },
            ],
            images: MockImageResourceInfo {
                total: 18,
                original: 18,
                replaced: 4,
                invalid: 0,
                unused_edits: 1,
            },
            staging_path: "D:/FPTU/novel-translate-cli/workspace/rezero/staging/v26".to_string(),
            edit_folder_path: "D:/FPTU/novel-translate-cli/workspace/rezero/images/edit"
                .to_string(),
            output_path: "D:/FPTU/novel-translate-cli/workspace/rezero/dist/ReZero_Vol26_VI.epub"
                .to_string(),
            last_validation_time: "10:15:00".to_string(),
            can_export: true,
            errors: vec![],
        };

        let logs = vec![
            MockLogEntry {
                id: 1,
                timestamp: "10:10:02".to_string(),
                level: LogLevel::Info,
                event: "Workspace Loaded".to_string(),
                volume: "Vol. 26".to_string(),
                chapter: "Ch. 04".to_string(),
                model: "-".to_string(),
                status: "Success".to_string(),
                message: "Workspace rezero-arc7 loaded with 4 volumes, active: Vol 26".to_string(),
            },
            MockLogEntry {
                id: 2,
                timestamp: "10:11:15".to_string(),
                level: LogLevel::Info,
                event: "Batch 01 Completed".to_string(),
                volume: "Vol. 26".to_string(),
                chapter: "Ch. 04".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                status: "Success".to_string(),
                message: "Translated blocks #B001-#B020. Structure validated 100%.".to_string(),
            },
            MockLogEntry {
                id: 3,
                timestamp: "10:12:40".to_string(),
                level: LogLevel::Info,
                event: "Batch 02 Completed".to_string(),
                volume: "Vol. 26".to_string(),
                chapter: "Ch. 04".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                status: "Success".to_string(),
                message: "Translated blocks #B021-#B040.".to_string(),
            },
            MockLogEntry {
                id: 4,
                timestamp: "10:13:05".to_string(),
                level: LogLevel::Warn,
                event: "Review Issues Flagged".to_string(),
                volume: "Vol. 26".to_string(),
                chapter: "Ch. 04".to_string(),
                model: "gpt-4o".to_string(),
                status: "Warning".to_string(),
                message: "Detected 3 issues: #001 (S2 Terminology), #002 (S1 Addressing), #003 (S3 Meaning).".to_string(),
            },
            MockLogEntry {
                id: 5,
                timestamp: "10:14:22".to_string(),
                level: LogLevel::Info,
                event: "Review Cycle Dispatched".to_string(),
                volume: "Vol. 26".to_string(),
                chapter: "Ch. 04".to_string(),
                model: "gpt-4o".to_string(),
                status: "Running".to_string(),
                message: "Issue #003 exceeded 5 auto-fix cycles. Marked for User Intervention.".to_string(),
            },
        ];

        Self {
            workspace,
            task,
            review_issues,
            editor_blocks,
            characters,
            terminology,
            addressing,
            relationships,
            timeline,
            skills,
            decisions,
            export,
            logs,
            active_scenario: DemoScenario::NormalTranslation,
        }
    }

    pub fn apply_scenario(&mut self, scenario: DemoScenario) {
        self.active_scenario = scenario;
        match scenario {
            DemoScenario::NormalTranslation => {
                self.task.is_running = true;
                self.task.is_paused = false;
                self.task.current_stage = WorkflowStage::TranslatingBatch;
                self.task.stage_states = [
                    StepState::Completed,
                    StepState::Running,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                ];
                self.task.active_model.model_name = "claude-3-5-sonnet".to_string();
                self.task.active_model.role = ModelRole::Translation;
                self.task.current_operation =
                    "Translating Japanese block batch #B041-#B050 with glossary context"
                        .to_string();
                self.export.can_export = true;
            }
            DemoScenario::ReviewIssuesDetected => {
                self.task.is_running = false;
                self.task.is_paused = false;
                self.task.current_stage = WorkflowStage::ReviewingBatch;
                self.task.stage_states = [
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Running,
                    StepState::Pending,
                    StepState::Pending,
                ];
                self.task.active_model.model_name = "gpt-4o".to_string();
                self.task.active_model.role = ModelRole::Review;
                self.task.current_operation =
                    "Reviewing batch #B041-#B050 against rubric S0-S3".to_string();
                self.export.can_export = false;
            }
            DemoScenario::WaitingForTerminologyApproval => {
                self.task.is_running = false;
                self.task.is_paused = true;
                self.task.current_stage = WorkflowStage::PreparingContext;
                self.task.stage_states = [
                    StepState::WaitingForUser,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                ];
                self.task.active_model.model_name = "gpt-4o-mini".to_string();
                self.task.active_model.role = ModelRole::Terminology;
                self.task.current_operation =
                    "Waiting for human approval on pending terminology candidates".to_string();
            }
            DemoScenario::ModelRequestFailed => {
                self.task.is_running = false;
                self.task.is_paused = true;
                self.task.current_stage = WorkflowStage::TranslatingBatch;
                self.task.stage_states = [
                    StepState::Completed,
                    StepState::Failed,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                    StepState::Pending,
                ];
                self.task.current_operation =
                    "Error 503: Upstream model provider temporarily unavailable. Backoff 5s."
                        .to_string();
                self.add_log(
                    LogLevel::Error,
                    "API Request Failed",
                    "HTTP 503 Service Unavailable on translation batch",
                );
            }
            DemoScenario::BudgetLimitReached => {
                self.task.is_running = false;
                self.task.is_paused = true;
                self.task.session_cost_usd = 5.00;
                self.task.budget_limit_usd = Some(5.00);
                self.task.budget_unlimited = false;
                self.task.current_operation =
                    "Budget limit reached ($5.00). Task paused to prevent unexpected costs."
                        .to_string();
                self.add_log(
                    LogLevel::Warn,
                    "Budget Cap Hit",
                    "Reached predefined cost limit ($5.00). Pausing runner.",
                );
            }
            DemoScenario::TranslationCompleted => {
                self.task.is_running = false;
                self.task.is_paused = false;
                self.task.volume_progress_pct = 100;
                self.task.chapter_progress_pct = 100;
                self.task.current_stage = WorkflowStage::FinalChapterReview;
                self.task.stage_states = [
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Completed,
                    StepState::Completed,
                ];
                self.task.current_operation =
                    "All selected chapters completed & verified. Ready for staging & export."
                        .to_string();
                for issue in &mut self.review_issues {
                    issue.resolved = true;
                }
                self.export.can_export = true;
                self.export.checklist[0].passed = true;
                self.export.checklist[1].passed = true;
                self.export.checklist[1].details =
                    "Tất cả các issue đã được giải quyết hoặc phê duyệt ngoại lệ".to_string();
                self.export.checklist[4].passed = true;
                self.export.errors.clear();
            }
            DemoScenario::ExportValidationFailed => {
                self.export.can_export = false;
                self.export.checklist[1].passed = false;
                self.export.checklist[1].details =
                    "Còn 3 vấn đề Review chưa được xử lý (S1, S2, S3)".to_string();
                self.export.checklist[4].passed = false;
                self.export.checklist[4].details =
                    "Lỗi cú pháp OPF: Thiếu tài nguyên ảnh minh họa cover.xhtml".to_string();
                self.export.images.invalid = 2;
                self.export.errors = vec![
                    "EPUBCheck Error: RSC-005 - Resource not found: OEBPS/images/ill_004.jpg"
                        .to_string(),
                    "Review Error: 3 open issues blocking final production export".to_string(),
                ];
                self.add_log(
                    LogLevel::Error,
                    "Export Validation Failed",
                    "EPUBCheck returned 1 critical error, 2 invalid image references",
                );
            }
        }
    }

    pub fn add_log(&mut self, level: LogLevel, event: &str, message: &str) {
        let id = self.logs.len() + 1;
        self.logs.push(MockLogEntry {
            id,
            timestamp: "10:16:30".to_string(),
            level,
            event: event.to_string(),
            volume: "Vol. 26".to_string(),
            chapter: "Ch. 04".to_string(),
            model: self.task.active_model.model_name.clone(),
            status: level.label().to_string(),
            message: message.to_string(),
        });
    }

    pub fn unresolved_issues_count(&self) -> usize {
        self.review_issues.iter().filter(|i| !i.resolved).count()
    }

    pub fn pending_decisions_count(&self) -> usize {
        self.decisions
            .iter()
            .filter(|d| !d.is_confirmed && !d.is_deferred)
            .count()
    }
}
