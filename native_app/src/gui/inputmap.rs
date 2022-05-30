use crate::input::{InputContext, KeyCode, MouseButton};
use common::{FastMap, FastSet};
use geom::Vec3;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};

// Either combinations can work
pub struct InputCombinations(pub(crate) Vec<InputCombination>);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum UnitInput {
    Key(KeyCode),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum InputAction {
    GoLeft,
    GoRight,
    GoForward,
    GoBackward,
    CameraMove,
    CameraRotate,
    Zoom,
    Dezoom,
    Rotate,
    Close,
    Select,
    NoSnapping,
    HideInterface,
}

// All unit inputs need to match
pub struct InputCombination(Vec<UnitInput>);

struct InputTree {
    actions: Vec<InputAction>,
    childs: FastMap<UnitInput, Box<InputTree>>,
}
pub struct InputMap {
    pub just_act: FastSet<InputAction>,
    pub act: FastSet<InputAction>,
    pub input_mapping: FastMap<InputAction, InputCombinations>,
    pub wheel: f32,
    pub unprojected: Option<Vec3>,
    input_tree: InputTree,
}

impl InputMap {
    #[rustfmt::skip]
    pub fn default_mapping() -> FastMap<InputAction, InputCombinations> {
        let mut m = FastMap::default();
        use InputAction::*;
        use UnitInput::*;
        use MouseButton::*;
        use KeyCode as K;

        macro_rules! ics {
            [$($($v:expr),+);+] => {
                InputCombinations(vec![$(InputCombination(vec![$($v),+])),+])
            }
        }

        for (k, v) in vec![
            (GoForward,     ics![Key(K::Z) ; Key(K::Up)]),
            (GoBackward,    ics![Key(K::S) ; Key(K::Down)]),
            (GoLeft,        ics![Key(K::Q) ; Key(K::Left)]),
            (GoRight,       ics![Key(K::D) ; Key(K::Right)]),
            (CameraMove,    ics![Mouse(Right)]),
            (CameraRotate,  ics![Key(K::LShift), Mouse(Right) ; Mouse(Middle)]),
            (Zoom,          ics![Key(K::Plus) ; WheelUp]),
            (Dezoom,        ics![Key(K::Minus) ; WheelDown]),
            (Rotate,        ics![Key(K::LControl), WheelUp ; Key(K::LControl), WheelDown]),
            (Close,         ics![Key(K::Escape) ; Mouse(Right)]),
            (Select,        ics![Mouse(Left)]),
            (NoSnapping,    ics![Key(K::LControl)]),
            (HideInterface, ics![Key(K::H)]),
        ] {
            if m.insert(k, v).is_some() {
                log::error!("inserting same action twice!");
            }
        }

        m
    }

    fn build_input_tree(&mut self) {
        for v in &mut self.input_mapping.values_mut() {
            for x in &mut v.0 {
                x.0.sort()
            }
        }
        self.input_tree = InputTree::new(&self.input_mapping);
    }

    pub fn prepare_frame(&mut self, input: &InputContext, kb: bool, mouse: bool) {
        self.just_act.clear();
        let empty1 = FastSet::default();
        let empty2 = FastSet::default();
        let mut acts: FastSet<_> = self
            .input_tree
            .query(
                if kb { &input.keyboard.pressed } else { &empty1 },
                if mouse { &input.mouse.pressed } else { &empty2 },
                if mouse { input.mouse.wheel_delta } else { 0.0 },
            )
            .collect();
        std::mem::swap(&mut self.act, &mut acts);
        for v in &self.act {
            if !acts.contains(v) {
                self.just_act.insert(*v);
            }
        }
        self.wheel = input.mouse.wheel_delta;
    }
}

impl Display for InputCombinations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            Display::fmt(x, f)?;
            if i < self.0.len() - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

impl Display for InputCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            Display::fmt(x, f)?;
            if i < self.0.len() - 1 {
                f.write_str(" + ")?;
            }
        }
        Ok(())
    }
}

impl Display for UnitInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitInput::Key(code) => Debug::fmt(code, f),
            UnitInput::Mouse(mb) => Debug::fmt(mb, f),
            UnitInput::WheelUp => {
                write!(f, "Scroll Up")
            }
            UnitInput::WheelDown => {
                write!(f, "Scroll Down")
            }
        }
    }
}

impl Debug for UnitInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputCombinations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Default for InputMap {
    fn default() -> Self {
        let mut s = Self {
            just_act: Default::default(),
            act: Default::default(),
            input_mapping: Self::default_mapping(),
            wheel: 0.0,
            unprojected: None,
            input_tree: InputTree {
                childs: Default::default(),
                actions: Vec::new(),
            },
        };
        s.build_input_tree();
        s
    }
}

impl InputTree {
    pub fn new(mapping: &FastMap<InputAction, InputCombinations>) -> Self {
        let mut root = Self {
            actions: Vec::new(),
            childs: Default::default(),
        };

        for (act, combs) in mapping {
            log::info!("{} {}", act, combs);
            for comb in &combs.0 {
                let mut cur: &mut InputTree = &mut root;
                for inp in &comb.0 {
                    let ent = cur.childs.entry(*inp);
                    match ent {
                        Entry::Occupied(_) => {}
                        Entry::Vacant(v) => {
                            v.insert(Box::new(InputTree {
                                actions: Vec::new(),
                                childs: Default::default(),
                            }));
                        }
                    }
                    cur = &mut **cur.childs.get_mut(inp).unwrap();
                }
                cur.actions.push(*act);
            }
        }

        root
    }

    pub fn query(
        &self,
        kb: &FastSet<KeyCode>,
        mouse: &FastSet<MouseButton>,
        wheel: f32,
    ) -> impl Iterator<Item = InputAction> + '_ {
        let mut units: HashSet<UnitInput> = HashSet::with_capacity(kb.len() + mouse.len() + 1);

        units.extend(kb.iter().map(|x| UnitInput::Key(*x)));
        units.extend(mouse.iter().map(|x| UnitInput::Mouse(*x)));
        if wheel > 0.0 {
            units.insert(UnitInput::WheelUp);
        }
        if wheel < 0.0 {
            units.insert(UnitInput::WheelDown);
        }

        let mut matches = vec![];
        let mut queue = vec![(vec![], self)];

        while !queue.is_empty() {
            for (input_stack, q) in std::mem::take(&mut queue) {
                for key in units.iter().copied() {
                    if let Some(inp) = q.childs.get(&key) {
                        let mut newstack = input_stack.clone();
                        newstack.push(key);
                        if !inp.actions.is_empty() {
                            matches.push((newstack.clone(), &inp.actions));
                        }
                        queue.push((newstack, &**inp));
                    }
                }
            }
        }

        matches
            .into_iter()
            .rev()
            .filter_map(move |(inp, act)| {
                if !inp.iter().all(|x| units.contains(x)) {
                    return None;
                }
                for v in inp {
                    units.remove(&v);
                }
                Some(act)
            })
            .flatten()
            .copied()
    }
}

impl Display for InputAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InputAction::GoLeft => "Go Left",
                InputAction::GoRight => "Go Right",
                InputAction::GoForward => "Go Forward",
                InputAction::GoBackward => "Go Backward",
                InputAction::CameraMove => "Camera Move",
                InputAction::CameraRotate => "Camera Rotate",
                InputAction::Zoom => "Zoom",
                InputAction::Dezoom => "Dezoom",
                InputAction::Rotate => "Rotate",
                InputAction::Close => "Close",
                InputAction::Select => "Select",
                InputAction::HideInterface => "Hide interface",
                InputAction::NoSnapping => "No Snapping",
            }
        )
    }
}
