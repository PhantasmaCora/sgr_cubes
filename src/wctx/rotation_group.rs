

use cgmath::Vector3;
use cgmath::Quaternion;
use cgmath::One;
use cgmath::InnerSpace;
use cgmath::AbsDiffEq;

pub enum RotType {
    Static,
    RotFace,
    RotVert,
    RotEdge,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum RotFace {
    PlusZ,
    MinusZ,
    PlusY,
    MinusY,
    PlusX,
    MinusX
}

pub fn rf_to_vector(rf: RotFace) -> Vector3<f32> {
    match rf {
        RotFace::PlusZ => Vector3::<f32>::unit_z(),
        RotFace::MinusZ => -1.0 * Vector3::<f32>::unit_z(),
        RotFace::PlusX => Vector3::<f32>::unit_x(),
        RotFace::MinusX => -1.0 * Vector3::<f32>::unit_x(),
        RotFace::PlusY => Vector3::<f32>::unit_y(),
        RotFace::MinusY => -1.0 * Vector3::<f32>::unit_y()
    }
}

pub fn reverse_rf(rf: RotFace) -> RotFace {
    match rf {
        RotFace::PlusZ => RotFace::MinusZ,
        RotFace::MinusZ => RotFace::PlusZ,
        RotFace::PlusX => RotFace::MinusX,
        RotFace::MinusX => RotFace::PlusX,
        RotFace::PlusY => RotFace::MinusY,
        RotFace::MinusY => RotFace::PlusY
    }
}

pub fn num_to_rf(num: u8) -> Option<RotFace> {
    match num {
        0 => Some(RotFace::PlusZ),
        1 => Some(RotFace::MinusZ),
        2 => Some(RotFace::PlusY),
        3 => Some(RotFace::MinusY),
        4 => Some(RotFace::PlusX),
        5 => Some(RotFace::MinusX),
        _ => None
    }
}


pub fn rf_to_num(rf: RotFace) -> u8 {
    match rf {
        RotFace::PlusZ => 0,
        RotFace::MinusZ => 1,
        RotFace::PlusX => 4,
        RotFace::MinusX => 5,
        RotFace::PlusY => 2,
        RotFace::MinusY => 3
    }
}

pub fn vector_to_rf( vect: Vector3<f32> ) -> Option<RotFace> {
    let mut vv = vect.normalize();

    if vv.abs_diff_eq( &Vector3::<f32>::unit_y(), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::PlusY); }
    if vv.abs_diff_eq( &(-1.0 * Vector3::<f32>::unit_y()), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::MinusY); }
    if vv.abs_diff_eq( &Vector3::<f32>::unit_z(), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::PlusZ); }
    if vv.abs_diff_eq( &(-1.0 * Vector3::<f32>::unit_z()), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::MinusZ); }
    if vv.abs_diff_eq( &Vector3::<f32>::unit_x(), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::PlusX); }
    if vv.abs_diff_eq( &(-1.0 * Vector3::<f32>::unit_x()), Vector3::<f32>::default_epsilon() ) { return Some(RotFace::MinusX); }

    None
}

pub fn rotate_rf( rf: RotFace, quat: &Quaternion<f32> ) -> Option<RotFace> {
    let v = rf_to_vector(rf);
    vector_to_rf( quat.normalize() * v )
}

pub fn generate_quat_from_rf( rf: RotFace ) -> Quaternion<f32> {
    let zero = rf_to_vector(RotFace::PlusZ).normalize();
    let input = rf_to_vector(rf).normalize();

    Quaternion::<f32>::from_arc( zero, input, Some(zero) ).normalize()
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum RotVert {
    XmYmZm, // minus x minus y minus z
    XmYmZp, // minus x minus y plus z
    XmYpZm, // minus x plus y minus z
    XmYpZp, // minus x plus y plus z
    XpYmZm, // plus x minus y minus z
    XpYmZp, // plus x minus y plus z
    XpYpZm, // plus x plus y minus z
    XpYpZp // plus x plus y plus z
}

pub fn reverse_rv( rv: RotVert ) -> RotVert {
    match rv {
        RotVert::XmYmZm => RotVert::XpYpZp,
        RotVert::XmYmZp => RotVert::XpYpZm,
        RotVert::XmYpZm => RotVert::XpYmZp,
        RotVert::XmYpZp => RotVert::XpYmZm,
        RotVert::XpYmZm => RotVert::XmYpZp,
        RotVert::XpYmZp => RotVert::XmYpZm,
        RotVert::XpYpZm => RotVert::XmYmZp,
        RotVert::XpYpZp => RotVert::XmYmZm
    }
}

pub fn num_to_rv(num: u8) -> Option<RotVert> {
    match num {
        0 => Some(RotVert::XmYmZm),
        1 => Some(RotVert::XmYmZp),
        2 => Some(RotVert::XmYpZm),
        3 => Some(RotVert::XmYpZp),
        4 => Some(RotVert::XpYmZm),
        5 => Some(RotVert::XpYmZp),
        6 => Some(RotVert::XpYpZm),
        7 => Some(RotVert::XpYpZp),
        _ => None
    }
}

pub fn rv_to_vector(rv: RotVert) -> Vector3<f32> {
    match rv {
        RotVert::XmYmZm => Vector3::<f32>::new( -1.0, -1.0, -1.0 ),
        RotVert::XmYmZp => Vector3::<f32>::new( -1.0, -1.0, 1.0 ),
        RotVert::XmYpZm => Vector3::<f32>::new( -1.0, 1.0, -1.0 ),
        RotVert::XmYpZp => Vector3::<f32>::new( -1.0, 1.0, 1.0 ),
        RotVert::XpYmZm => Vector3::<f32>::new( 1.0, -1.0, -1.0 ),
        RotVert::XpYmZp => Vector3::<f32>::new( 1.0, -1.0, 1.0 ),
        RotVert::XpYpZm => Vector3::<f32>::new( 1.0, 1.0, -1.0 ),
        RotVert::XpYpZp => Vector3::<f32>::new( 1.0, 1.0, 1.0 )
    }
}

pub fn vector_to_rv( vect: Vector3<f32> ) -> Option<RotVert> {
    let tup: &(f32, f32, f32) = vect.as_ref();
    match tup {
        (-1.0, -1.0, -1.0) => Some(RotVert::XmYmZm),
        (-1.0, -1.0, 1.0) => Some(RotVert::XmYmZp),
        (-1.0, 1.0, -1.0) => Some(RotVert::XmYpZm),
        (-1.0, 1.0, 1.0) => Some(RotVert::XmYpZp),
        (1.0, -1.0, -1.0) => Some(RotVert::XpYmZm),
        (1.0, -1.0, 1.0) => Some(RotVert::XpYmZp),
        (1.0, 1.0, -1.0) => Some(RotVert::XpYpZm),
        (1.0, 1.0, 1.0) => Some(RotVert::XpYpZp),
        _ => None
    }
}

pub fn rotate_rv( rv: RotVert, quat: &Quaternion<f32> ) -> Option<RotVert> {
    let v = rv_to_vector(rv);
    vector_to_rv( quat * v )
}

pub fn generate_quat_from_rv( rv: RotVert ) -> Quaternion<f32> {
    let zero = rv_to_vector(RotVert::XmYmZm).normalize();
    let input = rv_to_vector(rv).normalize();

    Quaternion::<f32>::from_arc( zero, input, Some(zero) ).normalize()
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum RotEdge {
    TopZm,
    TopZp,
    TopXm,
    TopXp,
    MidZmXm,
    MidZpXp,
    MidZpXm,
    MidZmXp,
    LowZm,
    LowZp,
    LowXm,
    LowXp
}

pub fn reverse_re( re: RotEdge ) -> RotEdge {
    match re {
        RotEdge::TopZm => RotEdge::LowZp,
        RotEdge::TopZp => RotEdge::LowZm,
        RotEdge::TopXm => RotEdge::LowXp,
        RotEdge::TopXp => RotEdge::LowXm,
        RotEdge::MidZmXm => RotEdge::MidZpXp,
        RotEdge::MidZpXp => RotEdge::MidZmXm,
        RotEdge::MidZpXm => RotEdge::MidZmXp,
        RotEdge::MidZmXp => RotEdge::MidZpXm,
        RotEdge::LowZm => RotEdge::TopZp,
        RotEdge::LowZp => RotEdge::TopZm,
        RotEdge::LowXm => RotEdge::TopXp,
        RotEdge::LowXp => RotEdge::TopXm
    }
}

pub fn num_to_re(num: u8) -> Option<RotEdge> {
    match num {
        0 => Some(RotEdge::LowZm),
        1 => Some(RotEdge::LowZp),
        2 => Some(RotEdge::LowXm),
        3 => Some(RotEdge::LowXp),
        4 => Some(RotEdge::MidZmXm),
        5 => Some(RotEdge::MidZpXp),
        6 => Some(RotEdge::MidZpXm),
        7 => Some(RotEdge::MidZmXp),
        8 => Some(RotEdge::TopZm),
        9 => Some(RotEdge::TopZp),
        10 => Some(RotEdge::TopXm),
        11 => Some(RotEdge::TopXp),
        _ => None
    }
}


pub fn re_to_vector( re: RotEdge ) -> Vector3<f32> {
    match re {
        RotEdge::TopZm => Vector3::<f32>::new(0.0, 1.0, -1.0),
        RotEdge::TopZp => Vector3::<f32>::new(0.0, 1.0, 1.0),
        RotEdge::TopXm => Vector3::<f32>::new(-1.0, 1.0, 0.0),
        RotEdge::TopXp => Vector3::<f32>::new(1.0, 1.0, 0.0),
        RotEdge::MidZmXm => Vector3::<f32>::new(-1.0, 0.0, -1.0),
        RotEdge::MidZpXp => Vector3::<f32>::new(1.0, 0.0, 1.0),
        RotEdge::MidZpXm => Vector3::<f32>::new(1.0, 0.0, -1.0),
        RotEdge::MidZmXp => Vector3::<f32>::new(-1.0, 0.0, 1.0),
        RotEdge::LowZm => Vector3::<f32>::new(0.0, -1.0, -1.0),
        RotEdge::LowZp => Vector3::<f32>::new(0.0, -1.0, 1.0),
        RotEdge::LowXm => Vector3::<f32>::new(-1.0, -1.0, 0.0),
        RotEdge::LowXp => Vector3::<f32>::new(1.0, -1.0, 0.0)
    }
}

pub fn vector_to_re( vect: Vector3<f32> ) -> Option<RotEdge> {
    let mut n_high: u8 = 0;
    if vect.y == 0.0 {
        n_high = 1;
    } else if vect.y < 0.0 {
        n_high = 2;
    }

    let mut n_low: u8 = 255;
    if n_high == 1 {
        if vect.x < 0.0 && vect.z < 0.0 {
            n_low = 0;
        } else if vect.x > 0.0 && vect.z > 0.0 {
            n_low = 1;
        } else if vect.z > 0.0 && vect.x < 0.0 {
            n_low = 2;
        } else if vect.z < 0.0 && vect.x > 0.0 {
            n_low = 3;
        }
    } else {
        if vect.x == 0.0 && vect.z < 0.0 {
            n_low = 0;
        } else if vect.x == 0.0 && vect.z > 0.0 {
            n_low = 1;
        } else if vect.z == 0.0 && vect.x > 0.0 {
            n_low = 2;
        } else if vect.z == 0.0 && vect.x < 0.0 {
            n_low = 3;
        }
    }

    match ( n_high, n_low ) {
        (0, 1) => Some(RotEdge::LowZp),
        (0, 0) => Some(RotEdge::LowZm),
        (0, 3) => Some(RotEdge::LowXp),
        (0, 2) => Some(RotEdge::LowXm),
        (1, 1) => Some(RotEdge::MidZpXp),
        (1, 0) => Some(RotEdge::MidZmXm),
        (1, 3) => Some(RotEdge::MidZmXp),
        (1, 2) => Some(RotEdge::MidZpXm),
        (2, 1) => Some(RotEdge::TopZp),
        (2, 0) => Some(RotEdge::TopZm),
        (2, 3) => Some(RotEdge::TopXp),
        (2, 2) => Some(RotEdge::TopXm),
        _ => None
    }
}

pub fn rotate_re( re: RotEdge, quat: &Quaternion<f32> ) -> Option<RotEdge> {
    let v = re_to_vector(re);
    vector_to_re( quat * v )
}

pub fn generate_quat_from_re( re: RotEdge ) -> Quaternion<f32> {
    let q = match re {
        RotEdge::TopZm => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(0.0, 1.0, 0.0), None ),
        RotEdge::TopZp => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(0.0, 1.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(0.0, 0.0, 1.0), None ),
        RotEdge::TopXm => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(0.0, 1.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(-1.0, 0.0, 0.0), None ),
        RotEdge::TopXp => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(0.0, 1.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(1.0, 0.0, 0.0), None ),
        RotEdge::MidZmXm => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(-1.0, 0.0, 0.0), None ),
        RotEdge::MidZpXp => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(-1.0, 0.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(0.0, 0.0, 1.0), None ),
        RotEdge::MidZpXm => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(1.0, 0.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, 1.0), Vector3::<f32>::new(0.0, 1.0, 0.0), None ),
        RotEdge::MidZmXp => Quaternion::from_arc( Vector3::<f32>::new(0.0, -1.0, 0.0), Vector3::<f32>::new(-1.0, 0.0, 0.0), None ) * Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, 1.0), Vector3::<f32>::new(0.0, -1.0, 0.0), None ),
        RotEdge::LowZm => Quaternion::one(),
        RotEdge::LowZp => Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(0.0, 0.0, 1.0), None ),
        RotEdge::LowXm => Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(-1.0, 0.0, 0.0), None ),
        RotEdge::LowXp => Quaternion::from_arc( Vector3::<f32>::new(0.0, 0.0, -1.0), Vector3::<f32>::new(1.0, 0.0, 0.0), None )
    };
    q.normalize()
}
