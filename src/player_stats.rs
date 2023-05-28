#[derive(Clone, Debug)]
pub struct Skill {
    name: String,
    description: String,
}

impl Skill {
    pub fn from_str(name: &str, description: &str) -> Self {
        Skill {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillsChain {
    skills: Vec<Skill>,
    n_learned: usize,
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
        let attack_skills = SkillsChain::new(vec![
            Skill::from_str("Attack 1", "This is attack 1"),
            Skill::from_str("Attack 2", "This is attack 2"),
            Skill::from_str("Attack 3", "This is attack 3"),
        ]);

        let durability_skills = SkillsChain::new(vec![
            Skill::from_str("Durability 1", "This is durability 1"),
            Skill::from_str("Durability 2", "This is durability 2"),
            Skill::from_str("Durability 3", "This is durability 3"),
        ]);

        let agility_skills = SkillsChain::new(vec![
            Skill::from_str("Agility 1", "This is agility 1"),
            Skill::from_str("Agility 2", "This is agility 2"),
            Skill::from_str("Agility 3", "This is agility 3"),
        ]);

        let light_skills = SkillsChain::new(vec![
            Skill::from_str("Light 1", "This is light 1"),
            Skill::from_str("Light 2", "This is light 2"),
            Skill::from_str("Light 3", "This is light 3"),
        ]);

        Self {
            level: 1,
            level_up_exp: 100,
            exp: 0,
            n_skill_points: 1,

            attack_skills,
            durability_skills,
            agility_skills,
            light_skills,
        }
    }

    pub fn get_exp_ratio(&self) -> f32 {
        self.exp as f32 / self.level_up_exp as f32
    }

    pub fn add_exp(&mut self, value: usize) {
        self.exp += value;
        if self.exp >= self.level_up_exp {
            self.exp -= self.level_up_exp;
            self.level += 1;
            self.n_skill_points += 1;
            self.level_up_exp = (self.level_up_exp as f32 * 1.3) as usize;
        }
    }
}
