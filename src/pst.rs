pub const PST_OPENING: [[i64; 64]; 6] = [
  [ // white pawn
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
     -5,   0,   5,  20,  20,   5,   0,  -5,
    -10,   0,  10,  30,  30,  10,   0, -10,
    -10,   0,   5,  20,  20,   5,   0, -10,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0, -20, -20,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0
  ], 
  [ // white knight
    -80, -40, -30, -30, -30, -30, -40, -80,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,   5,  15,  15,   5,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  30,  15,  15,  30,   5, -30,
    -40, -20,   0,  10,  10,   0, -20, -40,
    -80, -40, -30, -30, -30, -30, -40, -80,
  ], 
  [ // white bishop
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
  ], 
  [ // white rook
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0
  ], 
  [ // white queen
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20
  ], 
  [ // white king
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20
  ]
];

pub const PST_ENDING: [[i64; 64]; 6] = [
  [ // white pawn
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  30,  20,  20,  30,  50,  50,
     30,  30,  20,  10,  10,  20,  30,  30,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0
  ], 
  [ // white knight
    -80, -40, -30, -30, -30, -30, -40, -80,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  30,  30,  15,   5, -30,
    -30,   0,  15,  30,  30,  15,   0, -30,
    -30,   5,   5,  15,  15,   5,   5, -30,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -80, -40, -30, -30, -30, -30, -40, -80,
  ], 
  [ // white bishop
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,  10,  15,  15,  10,   0, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -15,  -5,   0,   0,   0,   0,  -5, -15,
    -25, -15, -10, -10, -10, -10, -15, -25,
  ], 
  [ // white rook
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0
  ], 
  [ // white queen
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20
  ], 
  [ // white king
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50
  ]
];
