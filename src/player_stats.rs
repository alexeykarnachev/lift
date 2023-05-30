#[derive(Clone, Copy, Debug)]
pub enum SkillEffectType {
    SetDamageMultiplier(f32),
    SetSplashDamagePenalty(f32),
    SetStaminaCostMultiplier(f32),
    SetReceivedDamageMultiplier(f32),
}

#[derive(Clone, Debug)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub effect: SkillEffectType,
}

impl Skill {
    pub fn from_str(
        name: &str,
        description: &str,
        effect: SkillEffectType,
    ) -> Self {
        Skill {
            name: name.to_string(),
            description: description.to_string(),
            effect,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillsChain {
    pub skills: Vec<Skill>,
    pub n_learned: usize,
}

impl SkillsChain {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self {
            skills,
            n_learned: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub level: usize,
    pub level_up_exp: usize,
    pub exp: usize,
    pub n_skill_points: usize,

    pub attack_skills: SkillsChain,
    pub durability_skills: SkillsChain,
    pub agility_skills: SkillsChain,
    pub light_skills: SkillsChain,
}

impl Stats {
    pub fn new() -> Self {
        use SkillEffectType::*;
        let attack_skills = SkillsChain::new(vec![
            Skill::from_str(
                "Attack 1",
                "This is attack 1",
                SetDamageMultiplier(1.3),
            ),
            Skill::from_str(
                "Attack 2",
                "This is attack 2",
                SetDamageMultiplier(1.6),
            ),
            Skill::from_str(
                "Attack 3",
                "This is attack 3",
                SetSplashDamagePenalty(0.5),
            ),
        ]);

        let durability_skills = SkillsChain::new(vec![
            Skill::from_str(
                "Durability 1",
                "This is durability 1",
                SetReceivedDamageMultiplier(0.8),
            ),
            Skill::from_str(
                "Durability 2",
                "This is durability 2",
                SetReceivedDamageMultiplier(0.6),
            ),
            Skill::from_str(
                "Durability 3",
                "This is durability 3",
                SetReceivedDamageMultiplier(0.4),
            ),
        ]);

        let agility_skills = SkillsChain::new(vec![
            Skill::from_str(
                "Agility 1",
                "This is agility 1",
                SetStaminaCostMultiplier(0.8),
            ),
            Skill::from_str(
                "Agility 2",
                "This is agility 2",
                SetStaminaCostMultiplier(0.6),
            ),
            Skill::from_str(
                "Agility 3",
                "This is agility 3",
                SetStaminaCostMultiplier(0.4),
            ),
        ]);

        let light_skills = SkillsChain::new(vec![
            Skill::from_str(
                "Light 1",
                "This is light 1",
                SetStaminaCostMultiplier(0.8),
            ),
            Skill::from_str(
                "Light 2",
                "This is light 2",
                SetStaminaCostMultiplier(0.6),
            ),
            Skill::from_str(
                "Light 3",
                "This is light 3",
                SetStaminaCostMultiplier(0.4),
            ),
        ]);

        Self {
            level: 1,
            level_up_exp: 50,
            exp: 0,
            n_skill_points: 3,

            attack_skills,
            durability_skills,
            agility_skills,
            light_skills,
        }
    }

    pub fn get_exp_ratio(&self) -> f32 {
        self.exp as f32 / self.level_up_exp as f32
    }

    pub fn add_exp(&mut self, value: usize) -> bool {
        self.exp += value;
        if self.exp >= self.level_up_exp {
            self.exp -= self.level_up_exp;
            self.level += 1;
            self.n_skill_points += 1;
            self.level_up_exp = (self.level_up_exp as f32 * 1.3) as usize;
            return true;
        }

        false
    }

    pub fn force_learn_next(&mut self, name: &str) -> SkillEffectType {
        let skills = self.get_skills_by_name(&name);
        let effect = skills.skills[skills.n_learned].effect;
        skills.n_learned += 1;
        self.n_skill_points -= 1;

        effect
    }

    pub fn get_skills_by_name(&mut self, name: &str) -> &mut SkillsChain {
        match name {
            "attack_skills" => &mut self.attack_skills,
            "durability_skills" => &mut self.durability_skills,
            "agility_skills" => &mut self.agility_skills,
            "light_skills" => &mut self.light_skills,
            _ => {
                panic!("Unhandled skills chain name: {:?}", name)
            }
        }
    }
}
