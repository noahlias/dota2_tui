#[derive(Clone, Copy)]
pub enum Language {
    En,
    Zh,
}

pub struct I18n {
    lang: Language,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn language_from_config(value: &str) -> Language {
        match value.to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "cn" => Language::Zh,
            _ => Language::En,
        }
    }

    pub fn is_zh(&self) -> bool {
        matches!(self.lang, Language::Zh)
    }

    pub fn title_results(&self) -> &str {
        match self.lang {
            Language::En => "Search Results",
            Language::Zh => "搜索结果",
        }
    }

    pub fn title_matches(&self) -> &str {
        match self.lang {
            Language::En => "Matches",
            Language::Zh => "比赛列表",
        }
    }

    pub fn title_match_detail(&self) -> &str {
        match self.lang {
            Language::En => "Match Detail",
            Language::Zh => "比赛详情",
        }
    }

    pub fn title_recent(&self) -> &str {
        match self.lang {
            Language::En => "Recent",
            Language::Zh => "最近搜索",
        }
    }

    pub fn title_profile(&self) -> &str {
        match self.lang {
            Language::En => "Profile",
            Language::Zh => "玩家信息",
        }
    }

    pub fn title_status(&self) -> &str {
        match self.lang {
            Language::En => "Status",
            Language::Zh => "状态",
        }
    }

    pub fn title_views(&self) -> &str {
        match self.lang {
            Language::En => "Views",
            Language::Zh => "视图",
        }
    }

    pub fn tab_overview(&self) -> &str {
        match self.lang {
            Language::En => "Overview",
            Language::Zh => "总览",
        }
    }

    pub fn tab_matches(&self) -> &str {
        match self.lang {
            Language::En => "Matches",
            Language::Zh => "比赛",
        }
    }

    pub fn tab_stats(&self) -> &str {
        match self.lang {
            Language::En => "Stats",
            Language::Zh => "统计",
        }
    }

    pub fn table_hero(&self) -> &str {
        match self.lang {
            Language::En => "Hero",
            Language::Zh => "英雄",
        }
    }

    pub fn table_player(&self) -> &str {
        match self.lang {
            Language::En => "Player",
            Language::Zh => "玩家",
        }
    }

    pub fn table_result(&self) -> &str {
        match self.lang {
            Language::En => "W/L",
            Language::Zh => "胜败",
        }
    }

    pub fn table_mode(&self) -> &str {
        match self.lang {
            Language::En => "Mode",
            Language::Zh => "模式",
        }
    }

    pub fn table_duration(&self) -> &str {
        match self.lang {
            Language::En => "Dur",
            Language::Zh => "时长",
        }
    }

    pub fn table_time(&self) -> &str {
        match self.lang {
            Language::En => "Time",
            Language::Zh => "时间",
        }
    }

    pub fn table_k(&self) -> &str {
        match self.lang {
            Language::En => "K",
            Language::Zh => "击杀",
        }
    }

    pub fn table_d(&self) -> &str {
        match self.lang {
            Language::En => "D",
            Language::Zh => "死亡",
        }
    }

    pub fn table_a(&self) -> &str {
        match self.lang {
            Language::En => "A",
            Language::Zh => "助攻",
        }
    }

    pub fn table_gpm(&self) -> &str {
        match self.lang {
            Language::En => "GPM",
            Language::Zh => "金钱",
        }
    }

    pub fn table_xpm(&self) -> &str {
        match self.lang {
            Language::En => "XPM",
            Language::Zh => "经验",
        }
    }

    pub fn table_net(&self) -> &str {
        match self.lang {
            Language::En => "NET",
            Language::Zh => "净值",
        }
    }

    pub fn table_items(&self) -> &str {
        match self.lang {
            Language::En => "Items",
            Language::Zh => "物品",
        }
    }

    pub fn result_win(&self) -> &str {
        match self.lang {
            Language::En => "W",
            Language::Zh => "胜",
        }
    }

    pub fn result_loss(&self) -> &str {
        match self.lang {
            Language::En => "L",
            Language::Zh => "败",
        }
    }

    pub fn time_now(&self) -> &str {
        match self.lang {
            Language::En => "now",
            Language::Zh => "刚刚",
        }
    }

    pub fn time_minutes(&self, value: i64) -> String {
        match self.lang {
            Language::En => format!("{value}m"),
            Language::Zh => format!("{value}分钟"),
        }
    }

    pub fn time_hours(&self, value: i64) -> String {
        match self.lang {
            Language::En => format!("{value}h"),
            Language::Zh => format!("{value}小时"),
        }
    }

    pub fn time_days(&self, value: i64) -> String {
        match self.lang {
            Language::En => format!("{value}d"),
            Language::Zh => format!("{value}天"),
        }
    }

    pub fn match_wait(&self) -> &str {
        match self.lang {
            Language::En => "Select a match and press Enter",
            Language::Zh => "选择一场比赛并回车查看",
        }
    }

    pub fn loading_detail(&self) -> &str {
        match self.lang {
            Language::En => "Loading match details...",
            Language::Zh => "加载比赛详情中...",
        }
    }


    pub fn no_recent(&self) -> &str {
        match self.lang {
            Language::En => "No recent searches",
            Language::Zh => "暂无搜索记录",
        }
    }

    pub fn keybind_title(&self) -> &str {
        match self.lang {
            Language::En => "Keybinds",
            Language::Zh => "快捷键",
        }
    }

    pub fn search_hint(&self) -> &str {
        match self.lang {
            Language::En => "Search by account_id or SteamID64 only.\nExample: 135664392",
            Language::Zh => "仅支持 account_id 或 SteamID64 搜索\n示例: 135664392",
        }
    }

    pub fn banner_subtitle(&self) -> &str {
        match self.lang {
            Language::En => "TUI data explorer powered by OpenDota",
            Language::Zh => "TUI 数据探索器 powered by OpenDota",
        }
    }

    pub fn input_search(&self, editing: bool) -> &str {
        match (self.lang, editing) {
            (Language::En, true) => "Search (editing)",
            (Language::En, false) => "Search (/)",
            (Language::Zh, true) => "搜索（输入中）",
            (Language::Zh, false) => "搜索（/）",
        }
    }

    pub fn title_quick_stats(&self) -> &str {
        match self.lang {
            Language::En => "Quick stats",
            Language::Zh => "快速统计",
        }
    }

    pub fn title_avatar(&self) -> &str {
        match self.lang {
            Language::En => "Avatar",
            Language::Zh => "头像",
        }
    }

    pub fn title_loadout(&self) -> &str {
        match self.lang {
            Language::En => "Loadout",
            Language::Zh => "装备",
        }
    }

    pub fn title_hints(&self) -> &str {
        match self.lang {
            Language::En => "Hints",
            Language::Zh => "提示",
        }
    }

    pub fn hint_tabs(&self) -> &str {
        match self.lang {
            Language::En => "Use arrow keys to switch tabs",
            Language::Zh => "使用左右方向键切换标签",
        }
    }

    pub fn title_winrate(&self) -> &str {
        match self.lang {
            Language::En => "Winrate",
            Language::Zh => "胜率",
        }
    }

    pub fn title_recent_results(&self) -> &str {
        match self.lang {
            Language::En => "Recent results",
            Language::Zh => "近期结果",
        }
    }

    pub fn title_summary(&self) -> &str {
        match self.lang {
            Language::En => "Summary",
            Language::Zh => "汇总",
        }
    }

    pub fn label_name(&self) -> &str {
        match self.lang {
            Language::En => "Name",
            Language::Zh => "昵称",
        }
    }

    pub fn label_steamid(&self) -> &str {
        match self.lang {
            Language::En => "SteamID64",
            Language::Zh => "SteamID64",
        }
    }

    pub fn label_mmr(&self) -> &str {
        match self.lang {
            Language::En => "MMR estimate",
            Language::Zh => "预估分数",
        }
    }

    pub fn unknown(&self) -> &str {
        match self.lang {
            Language::En => "Unknown",
            Language::Zh => "未知",
        }
    }

    pub fn placeholder_dash(&self) -> &str {
        "-"
    }

    pub fn status_ready(&self) -> &str {
        match self.lang {
            Language::En => "Press / to search by SteamID64 or account_id",
            Language::Zh => "按 / 输入 SteamID64 或 account_id 搜索",
        }
    }

    pub fn loading_player(&self) -> &str {
        match self.lang {
            Language::En => "Loading...",
            Language::Zh => "加载中...",
        }
    }

    pub fn no_player_loaded(&self) -> &str {
        match self.lang {
            Language::En => "No player loaded",
            Language::Zh => "未加载玩家",
        }
    }

    pub fn quick_stats_format(&self, total: usize, wins: usize) -> String {
        match self.lang {
            Language::En => format!(
                "Recent matches: {total}\nWins: {wins}\nLosses: {}",
                total.saturating_sub(wins)
            ),
            Language::Zh => format!(
                "最近比赛: {total}\n胜场: {wins}\n败场: {}",
                total.saturating_sub(wins)
            ),
        }
    }

    pub fn stats_summary_format(&self, total: usize, wins: usize, winrate: f64) -> String {
        match self.lang {
            Language::En => {
                format!("Total: {total}\nWins: {wins}\nWinrate: {winrate:.1}%")
            }
            Language::Zh => {
                format!("总场次: {total}\n胜场: {wins}\n胜率: {winrate:.1}%")
            }
        }
    }

    pub fn status_search_cancelled(&self) -> &str {
        match self.lang {
            Language::En => "Search cancelled",
            Language::Zh => "已取消搜索",
        }
    }

    pub fn status_need_id(&self) -> &str {
        match self.lang {
            Language::En => "Enter a SteamID64 or account_id",
            Language::Zh => "请输入 SteamID64 或 account_id",
        }
    }

    pub fn status_invalid_id(&self) -> &str {
        match self.lang {
            Language::En => "Use account_id or SteamID64",
            Language::Zh => "仅支持 account_id 或 SteamID64",
        }
    }

    pub fn status_loading_player(&self, account_id: u32) -> String {
        match self.lang {
            Language::En => format!("Loading player {account_id}..."),
            Language::Zh => format!("加载玩家 {account_id}..."),
        }
    }

    pub fn status_loading_match(&self, match_id: u64) -> String {
        match self.lang {
            Language::En => format!("Loading match {match_id}..."),
            Language::Zh => format!("加载比赛 {match_id}..."),
        }
    }

    pub fn status_hero_loaded(&self) -> &str {
        match self.lang {
            Language::En => "Hero data loaded",
            Language::Zh => "英雄数据已加载",
        }
    }

    pub fn status_hero_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Hero load failed: {err}"),
            Language::Zh => format!("英雄数据加载失败: {err}"),
        }
    }

    pub fn status_match_loaded(&self) -> &str {
        match self.lang {
            Language::En => "Match details loaded",
            Language::Zh => "比赛详情已加载",
        }
    }

    pub fn status_match_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Match load failed: {err}"),
            Language::Zh => format!("比赛加载失败: {err}"),
        }
    }

    pub fn status_search_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Search failed: {err}"),
            Language::Zh => format!("搜索失败: {err}"),
        }
    }

    pub fn status_matches_loaded(&self) -> &str {
        match self.lang {
            Language::En => "Matches loaded. Use j/k and Enter for details",
            Language::Zh => "比赛列表已加载，使用 j/k 和回车查看详情",
        }
    }

    pub fn status_no_matches(&self) -> &str {
        match self.lang {
            Language::En => "No matches found",
            Language::Zh => "未找到比赛",
        }
    }

    pub fn status_matches_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Matches load failed: {err}"),
            Language::Zh => format!("比赛列表加载失败: {err}"),
        }
    }

    pub fn status_profile_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Profile load failed: {err}"),
            Language::Zh => format!("玩家信息加载失败: {err}"),
        }
    }

    pub fn status_image_failed(&self, err: &str) -> String {
        match self.lang {
            Language::En => format!("Image load failed: {err}"),
            Language::Zh => format!("图片加载失败: {err}"),
        }
    }

    pub fn help_labels(&self) -> [&str; 10] {
        match self.lang {
            Language::En => [
                "Search",
                "Quit",
                "Up/Down",
                "Select",
                "Top/Bottom",
                "Clear input",
                "Tab next/prev",
                "Help",
                "Keybinds",
                "Example",
            ],
            Language::Zh => [
                "搜索",
                "退出",
                "上/下",
                "选择",
                "顶部/底部",
                "清空输入",
                "切换标签",
                "帮助",
                "快捷键",
                "示例",
            ],
        }
    }

    pub fn help_group_search(&self) -> &str {
        match self.lang {
            Language::En => "Search",
            Language::Zh => "搜索",
        }
    }

    pub fn help_group_navigation(&self) -> &str {
        match self.lang {
            Language::En => "Navigation",
            Language::Zh => "导航",
        }
    }

    pub fn help_group_views(&self) -> &str {
        match self.lang {
            Language::En => "Views",
            Language::Zh => "视图",
        }
    }

    pub fn help_group_misc(&self) -> &str {
        match self.lang {
            Language::En => "General",
            Language::Zh => "通用",
        }
    }

    pub fn title_radiant(&self) -> &str {
        match self.lang {
            Language::En => "Radiant",
            Language::Zh => "天辉",
        }
    }

    pub fn title_dire(&self) -> &str {
        match self.lang {
            Language::En => "Dire",
            Language::Zh => "夜魇",
        }
    }

    pub fn anonymous(&self) -> &str {
        match self.lang {
            Language::En => "Anonymous",
            Language::Zh => "匿名",
        }
    }

    pub fn format_game_mode(&self, game_mode: Option<i32>) -> String {
        match game_mode {
            Some(0) => if self.is_zh() { "未知".to_string() } else { "Unknown".to_string() },
            Some(1) => if self.is_zh() { "全英雄选择".to_string() } else { "All Pick".to_string() },
            Some(2) => if self.is_zh() { "队长模式".to_string() } else { "Captains Mode".to_string() },
            Some(3) => if self.is_zh() { "随机征召".to_string() } else { "Random Draft".to_string() },
            Some(4) => if self.is_zh() { "单一征召".to_string() } else { "Single Draft".to_string() },
            Some(5) => if self.is_zh() { "全随机".to_string() } else { "All Random".to_string() },
            Some(6) => if self.is_zh() { "新手教程".to_string() } else { "Intro".to_string() },
            Some(7) => if self.is_zh() { "Diretide".to_string() } else { "Diretide".to_string() },
            Some(8) => if self.is_zh() { "反向队长".to_string() } else { "Reverse Captains Mode".to_string() },
            Some(9) => if self.is_zh() { "Greeviling".to_string() } else { "Greeviling".to_string() },
            Some(10) => if self.is_zh() { "教程".to_string() } else { "Tutorial".to_string() },
            Some(11) => if self.is_zh() { "中路1v1".to_string() } else { "Mid Only".to_string() },
            Some(12) => if self.is_zh() { "最少使用".to_string() } else { "Least Played".to_string() },
            Some(13) => if self.is_zh() { "限制英雄".to_string() } else { "Limited Heroes".to_string() },
            Some(14) => if self.is_zh() { "宝典匹配".to_string() } else { "Compendium Matchmaking".to_string() },
            Some(15) => if self.is_zh() { "自定义".to_string() } else { "Custom".to_string() },
            Some(16) => if self.is_zh() { "队长征召".to_string() } else { "Captains Draft".to_string() },
            Some(17) => if self.is_zh() { "平衡征召".to_string() } else { "Balanced Draft".to_string() },
            Some(18) => if self.is_zh() { "技能征召".to_string() } else { "Ability Draft".to_string() },
            Some(19) => if self.is_zh() { "活动".to_string() } else { "Event".to_string() },
            Some(20) => if self.is_zh() { "全随机死斗".to_string() } else { "All Random Death Match".to_string() },
            Some(21) => if self.is_zh() { "1v1 中路".to_string() } else { "1v1 Mid".to_string() },
            Some(22) => if self.is_zh() { "全征召".to_string() } else { "All Draft".to_string() },
            Some(23) => if self.is_zh() { "极速".to_string() } else { "Turbo".to_string() },
            Some(24) => if self.is_zh() { "变异".to_string() } else { "Mutation".to_string() },
            Some(25) => if self.is_zh() { "教练挑战".to_string() } else { "Coaches Challenge".to_string() },
            Some(value) => value.to_string(),
            None => self.placeholder_dash().to_string(),
        }
    }
}
