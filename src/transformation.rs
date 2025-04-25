fn matmul4x4(d: [i32; 4], t: [[i32; 4]; 4]) -> [i32; 4] {
    [
        d[0] * t[0][0] + d[1] * t[0][1] + d[2] * t[0][2] + d[3] * t[0][3],
        d[0] * t[1][0] + d[1] * t[1][1] + d[2] * t[1][2] + d[3] * t[1][3],
        d[0] * t[2][0] + d[1] * t[2][1] + d[2] * t[2][2] + d[3] * t[2][3],
        d[0] * t[3][0] + d[1] * t[3][1] + d[2] * t[3][2] + d[3] * t[3][3],
    ]
}

fn matdif4x1(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    [b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]]
}

fn transform(start: [i32; 4], end: [i32; 4]) -> [i32; 4] {
    let d = matdif4x1(start, end);
    let t: [[i32; 4]; 4] = [
        [1, 1, 1, 1],   // total motion
        [1, -1, -1, 1], // x contrary
        [1, -1, 1, -1], // y contrary
        [1, 1, -1, -1], // z contrary
    ];
    return matmul4x4(d, t);
}

pub fn convert(voice_leadings: Vec<[i32; 4]>) -> Vec<[i32; 4]> {
    let mut out: Vec<[i32; 4]> = Vec::<[i32; 4]>::new();
    for i in 0..(voice_leadings.len() - 1) {
        let cur = voice_leadings[i];
        let next = voice_leadings[i + 1];
        out.push(transform(cur, next));
    }
    return out;
}
