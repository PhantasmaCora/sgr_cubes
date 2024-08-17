


use cgmath::Vector3;

pub enum RotType {
    RotFace,
    RotVert,
    RotEdge,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum RotFace {
    PlusY,
    MinusY,
    PlusZ,
    MinusZ,
    PlusX,
    MinusX
}

pub fn rf_to_vector(rf: RotFace) -> Vector3<i32> {
    match rf {
        RotFace::PlusZ => Vector3::<i32>::unit_z(),
        RotFace::MinusZ => -1 * Vector3::<i32>::unit_z(),
        RotFace::PlusX => Vector3::<i32>::unit_x(),
        RotFace::MinusX => -1 * Vector3::<i32>::unit_x(),
        RotFace::PlusY => Vector3::<i32>::unit_y(),
        RotFace::MinusY => -1 * Vector3::<i32>::unit_y()
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

pub fn vector_to_rf( vect: Vector3<i32> ) -> Option<RotFace> {
    if vect == Vector3::<i32>::unit_y() { return Some(RotFace::PlusY); }
    if vect == -1 * Vector3::<i32>::unit_y() { return Some(RotFace::MinusY); }
    if vect == Vector3::<i32>::unit_z() { return Some(RotFace::PlusZ); }
    if vect == -1 * Vector3::<i32>::unit_z() { return Some(RotFace::MinusZ); }
    if vect == Vector3::<i32>::unit_x() { return Some(RotFace::PlusX); }
    if vect == -1 * Vector3::<i32>::unit_x() { return Some(RotFace::MinusX); }

    None
}

pub fn rotate_rf( rf: RotFace, quat: &cgmath::Quaternion<f32> ) -> Option<RotFace> {
    let v: Vector3<f32> = i_to_f( rf_to_vector(rf) );
    vector_to_rf( f_to_i( quat * v ) )
}

#[derive(Copy, Clone)]
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

pub fn rv_to_vector(rv: RotVert) -> Vector3<i32> {
    match rv {
        RotVert::XmYmZm => Vector3::<i32>::new( -1, -1, -1 ),
        RotVert::XmYmZp => Vector3::<i32>::new( -1, -1, 1 ),
        RotVert::XmYpZm => Vector3::<i32>::new( -1, 1, -1 ),
        RotVert::XmYpZp => Vector3::<i32>::new( -1, 1, 1 ),
        RotVert::XpYmZm => Vector3::<i32>::new( 1, -1, -1 ),
        RotVert::XpYmZp => Vector3::<i32>::new( 1, -1, 1 ),
        RotVert::XpYpZm => Vector3::<i32>::new( 1, 1, -1 ),
        RotVert::XpYpZp => Vector3::<i32>::new( 1, 1, 1 )
    }
}

pub fn vector_to_rv( vect: &Vector3<i32> ) -> Option<RotVert> {
    let tup: &(i32, i32, i32) = vect.as_ref();
    match tup {
        (-1, -1, -1) => Some(RotVert::XmYmZm),
        (-1, -1, 1) => Some(RotVert::XmYmZp),
        (-1, 1, -1) => Some(RotVert::XmYpZm),
        (-1, 1, 1) => Some(RotVert::XmYpZp),
        (1, -1, -1) => Some(RotVert::XpYmZm),
        (1, -1, 1) => Some(RotVert::XpYmZp),
        (1, 1, -1) => Some(RotVert::XpYpZm),
        (1, 1, 1) => Some(RotVert::XpYpZp),
        _ => None
    }
}

pub fn rotate_rv( rv: RotVert, quat: &cgmath::Quaternion<f32> ) -> Option<RotVert> {
    let v: Vector3<f32> = i_to_f( rv_to_vector(rv) );
    vector_to_rv( &f_to_i( quat * v ) )
}

#[derive(Copy, Clone)]
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

pub fn re_to_vector( re: RotEdge ) -> Vector3<i32> {
    match re {
        RotEdge::TopZm => Vector3::<i32>::new(0, 1, -1),
        RotEdge::TopZp => Vector3::<i32>::new(0, 1, 1),
        RotEdge::TopXm => Vector3::<i32>::new(-1, 1, 0),
        RotEdge::TopXp => Vector3::<i32>::new(1, 1, 0),
        RotEdge::MidZmXm => Vector3::<i32>::new(-1, 0, -1),
        RotEdge::MidZpXp => Vector3::<i32>::new(1, 0, 1),
        RotEdge::MidZpXm => Vector3::<i32>::new(1, 0, -1),
        RotEdge::MidZmXp => Vector3::<i32>::new(-1, 0, 1),
        RotEdge::LowZm => Vector3::<i32>::new(0, -1, -1),
        RotEdge::LowZp => Vector3::<i32>::new(0, -1, 1),
        RotEdge::LowXm => Vector3::<i32>::new(-1, -1, 0),
        RotEdge::LowXp => Vector3::<i32>::new(1, -1, 0)
    }
}

pub fn vector_to_re( vect: &Vector3<i32> ) -> Option<RotEdge> {
    let mut n_high: u8 = 0;
    if vect.y == 0 {
        n_high = 1;
    } else if vect.y < 0 {
        n_high = 2;
    }

    let mut n_low: u8 = 255;
    if n_high == 1 {
        if vect.x < 0 && vect.z < 0 {
            n_low = 0;
        } else if vect.x > 0 && vect.z > 0 {
            n_low = 1;
        } else if vect.z > 0 && vect.x < 0{
            n_low = 2;
        } else if vect.z < 0 && vect.x > 0 {
            n_low = 3;
        }
    } else {
        if vect.x == 0 && vect.z < 0 {
            n_low = 0;
        } else if vect.x == 0 && vect.z > 0 {
            n_low = 1;
        } else if vect.z == 0 && vect.x > 0{
            n_low = 2;
        } else if vect.z == 0 && vect.x < 0 {
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

pub fn rotate_re( re: RotEdge, quat: &cgmath::Quaternion<f32> ) -> Option<RotEdge> {
    let v: Vector3<f32> = i_to_f( re_to_vector(re) );
    vector_to_re( &f_to_i( quat * v ) )
}

fn i_to_f( vect: Vector3<i32> ) -> Vector3<f32> {
    Vector3::<f32>::new( vect.x as f32, vect.y as f32, vect.z as f32 )
}

fn f_to_i( vect: Vector3<f32> ) -> Vector3<i32> {
    Vector3::<i32>::new( vect.x as i32, vect.y as i32, vect.z as i32 )
}
