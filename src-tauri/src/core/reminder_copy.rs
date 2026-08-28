use serde::{Deserialize, Serialize};

use crate::model::{BodyCounters, Tone};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Builtin {
    Stand,
    Water,
    Eyes,
    Review,
}

pub const ALL: [Builtin; 4] = [
    Builtin::Stand,
    Builtin::Water,
    Builtin::Eyes,
    Builtin::Review,
];

pub fn name(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "站起来动一动",
        Builtin::Water => "喝水",
        Builtin::Eyes => "远眺护眼",
        Builtin::Review => "收工前复盘",
    }
}

pub fn color(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "oklch(0.63 0.13 40)",
        Builtin::Water => "oklch(0.66 0.09 195)",
        Builtin::Eyes => "oklch(0.7 0.1 145)",
        Builtin::Review => "oklch(0.68 0.1 300)",
    }
}

pub fn detail(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "每 45 分钟 · 宠物提示 · 工作时段",
        Builtin::Water => "每 30 分钟 · 轻量气泡 · 计入每日 8 杯",
        Builtin::Eyes => "每 20 分钟 · 轻量气泡 · 20-20-20",
        Builtin::Review => "每天 17:30 · 全屏 · 仅工作日",
    }
}

fn pick(
    tone: Tone,
    professional: &'static str,
    gentle: &'static str,
    playful: &'static str,
) -> &'static str {
    match tone {
        Tone::Professional => professional,
        Tone::Gentle => gentle,
        Tone::Playful => playful,
    }
}

pub fn message(b: Builtin, tone: Tone) -> &'static str {
    match b {
        Builtin::Stand => pick(
            tone,
            "已连续坐着 45 分钟，请起身活动 2 分钟。",
            "坐久了，陪我一起站起来伸个懒腰？",
            "再坐下去你就要跟椅子长在一起了，起来！",
        ),
        Builtin::Water => pick(
            tone,
            "补充 200ml 水，今日 {cups}/{goal} 杯。",
            "喝口水吧，今天第 {next} 杯了。",
            "你的杯子在喊你，它说它很空。",
        ),
        Builtin::Eyes => pick(
            tone,
            "看向 6 米外物体并保持 20 秒。",
            "抬头看看窗外，20 秒就好。",
            "眼睛快冒烟了，看看远方压压火。",
        ),
        Builtin::Review => pick(
            tone,
            "用 5 分钟复盘今天并规划明天。",
            "收工前，和我一起理一理今天？",
            "先夸自己一句，再写下明天要干的事。",
        ),
    }
}

/// The 「01 从模板抓一个」 chips. Three of them are the seeded builtins under a
/// shorter name; the other three are templates of their own and carry their
/// own first line so the chip produces something that actually rings.
pub fn template_builtin(name: &str) -> Option<Builtin> {
    match name {
        "站立" => Some(Builtin::Stand),
        "喝水" => Some(Builtin::Water),
        "护眼" => Some(Builtin::Eyes),
        _ => None,
    }
}

/// What a reminder is painted when nobody said otherwise (＋ 空白).
pub const DEFAULT_COLOR: &str = "oklch(0.63 0.13 40)";

pub fn template_message(name: &str, tone: Tone) -> Option<&'static str> {
    Some(match name {
        "深呼吸" => pick(
            tone,
            "做 3 次深呼吸：吸气 4 秒，呼气 6 秒。",
            "停一下，跟我一起深呼吸三次？",
            "吸——呼——别憋着，我在数呢。",
        ),
        "肩颈拉伸" => pick(
            tone,
            "活动肩颈 1 分钟，左右各转 5 圈。",
            "肩膀酸了吧，转一转再继续。",
            "你的脖子在向我求救，扭一扭。",
        ),
        "记一句想法" => pick(
            tone,
            "用一句话记录当前的想法或进展。",
            "把刚才闪过的念头写下来吧。",
            "灵感不记就跑了，快写一句！",
        ),
        _ => return None,
    })
}

/// Placeholders a message may carry, filled in at fire time from the day's
/// body counters so the pet quotes real numbers rather than the design's
/// sample ones. Unknown braces are left alone.
pub const PLACEHOLDERS: [(&str, &str); 5] = [
    ("{cups}", "今日已喝杯数"),
    ("{goal}", "每日目标杯数"),
    ("{next}", "下一杯是第几杯"),
    ("{stands}", "今日站起次数"),
    ("{standGoal}", "每日站起目标"),
];

pub fn fill(message: &str, body: &BodyCounters) -> String {
    if !message.contains('{') {
        return message.to_string();
    }
    message
        .replace("{cups}", &body.water_cups.to_string())
        .replace("{goal}", &body.water_goal.to_string())
        .replace("{next}", &(body.water_cups + 1).to_string())
        .replace("{stands}", &body.stands.to_string())
        .replace("{standGoal}", &body.stand_goal.to_string())
}

pub fn hint(b: Builtin, tone: Tone) -> &'static str {
    match b {
        Builtin::Stand => pick(
            tone,
            "专注进行中时会推迟到本轮结束。",
            "我会等你这轮结束再叫你。",
            "我不打断你，但下课钟一响我就扑上来。",
        ),
        Builtin::Water => pick(
            tone,
            "菜单栏会累计今日饮水杯数。",
            "我帮你数着杯数。",
            "我偷偷在小本本上记你喝了几杯。",
        ),
        Builtin::Eyes => pick(
            tone,
            "遵循 20-20-20 护眼规则。",
            "20 分钟、20 英尺、20 秒。",
            "我数到 20 就放你走，说好了。",
        ),
        Builtin::Review => pick(
            tone,
            "自定义提醒：时间、文案、方式都可改。",
            "这条完全是你自己写的。",
            "这条是你自己加的，别怪我。",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BodyCounters, Tone};

    #[test]
    fn stand_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Stand, Tone::Professional),
            "已连续坐着 45 分钟，请起身活动 2 分钟。"
        );
        assert_eq!(
            message(Builtin::Stand, Tone::Gentle),
            "坐久了，陪我一起站起来伸个懒腰？"
        );
        assert_eq!(
            message(Builtin::Stand, Tone::Playful),
            "再坐下去你就要跟椅子长在一起了，起来！"
        );
    }

    fn body(cups: u32) -> BodyCounters {
        BodyCounters {
            water_cups: cups,
            water_goal: 8,
            stands: 2,
            stand_goal: 6,
            longest_sit_mins: 0,
            sit_goal_mins: 90,
            sit_secs: 0,
            day: String::new(),
        }
    }

    #[test]
    fn water_renders_to_the_spec_copy_for_its_sample_counts() {
        assert_eq!(
            fill(message(Builtin::Water, Tone::Professional), &body(6)),
            "补充 200ml 水，今日 6/8 杯。"
        );
        assert_eq!(
            fill(message(Builtin::Water, Tone::Gentle), &body(6)),
            "喝口水吧，今天第 7 杯了。"
        );
    }

    #[test]
    fn fill_leaves_plain_and_unknown_text_alone() {
        assert_eq!(fill("起来！", &body(0)), "起来！");
        assert_eq!(fill("{nope} {stands}/{standGoal}", &body(0)), "{nope} 2/6");
    }

    #[test]
    fn water_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Water, Tone::Professional),
            "补充 200ml 水，今日 {cups}/{goal} 杯。"
        );
        assert_eq!(
            message(Builtin::Water, Tone::Gentle),
            "喝口水吧，今天第 {next} 杯了。"
        );
        assert_eq!(
            message(Builtin::Water, Tone::Playful),
            "你的杯子在喊你，它说它很空。"
        );
    }

    #[test]
    fn eyes_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Eyes, Tone::Professional),
            "看向 6 米外物体并保持 20 秒。"
        );
        assert_eq!(
            message(Builtin::Eyes, Tone::Gentle),
            "抬头看看窗外，20 秒就好。"
        );
        assert_eq!(
            message(Builtin::Eyes, Tone::Playful),
            "眼睛快冒烟了，看看远方压压火。"
        );
    }

    #[test]
    fn review_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Review, Tone::Professional),
            "用 5 分钟复盘今天并规划明天。"
        );
        assert_eq!(
            message(Builtin::Review, Tone::Gentle),
            "收工前，和我一起理一理今天？"
        );
        assert_eq!(
            message(Builtin::Review, Tone::Playful),
            "先夸自己一句，再写下明天要干的事。"
        );
    }

    #[test]
    fn every_builtin_has_three_distinct_hints() {
        for b in ALL {
            let a = hint(b, Tone::Professional);
            let g = hint(b, Tone::Gentle);
            let p = hint(b, Tone::Playful);
            assert_ne!(a, g);
            assert_ne!(g, p);
            assert_ne!(a, p);
        }
    }

    #[test]
    fn hints_match_the_spec_for_stand() {
        assert_eq!(
            hint(Builtin::Stand, Tone::Professional),
            "专注进行中时会推迟到本轮结束。"
        );
        assert_eq!(
            hint(Builtin::Stand, Tone::Gentle),
            "我会等你这轮结束再叫你。"
        );
        assert_eq!(
            hint(Builtin::Stand, Tone::Playful),
            "我不打断你，但下课钟一响我就扑上来。"
        );
    }

    #[test]
    fn template_chips_resolve_to_builtins_or_their_own_copy() {
        assert_eq!(template_builtin("喝水"), Some(Builtin::Water));
        assert_eq!(template_builtin("深呼吸"), None);
        for name in ["深呼吸", "肩颈拉伸", "记一句想法"] {
            let a = template_message(name, Tone::Professional).unwrap();
            let g = template_message(name, Tone::Gentle).unwrap();
            let p = template_message(name, Tone::Playful).unwrap();
            assert_ne!(a, g);
            assert_ne!(g, p);
        }
        assert_eq!(template_message("站立", Tone::Playful), None);
    }

    #[test]
    fn names_details_and_colors_match_the_spec() {
        assert_eq!(name(Builtin::Stand), "站起来动一动");
        assert_eq!(name(Builtin::Water), "喝水");
        assert_eq!(name(Builtin::Eyes), "远眺护眼");
        assert_eq!(name(Builtin::Review), "收工前复盘");

        assert_eq!(color(Builtin::Water), "oklch(0.66 0.09 195)");
        assert_eq!(detail(Builtin::Eyes), "每 20 分钟 · 轻量气泡 · 20-20-20");
        assert_eq!(detail(Builtin::Review), "每天 17:30 · 全屏 · 仅工作日");
    }
}
