use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{Chip, Layouter, AssignedCell, Region, Value, Error},
    plonk::{Advice, Column, ConstraintSystem, Instance, Selector, Expression, VirtualCells},
    poly::Rotation,
};
use crate::prover::number::Number;


// round constants
ROUND_CONSTANTS = [
    [24448666467656506447555018649749346340705294023832615387641453784702583464707, 19752610610343814834081989345964253902282700341539483876504601969121084774539, 9520793415506326549109545537894287560752519598132096386048093015534488804808],  // round 0
    [22814234098357034097599682726494820560934925862581927123816510593532324971186, 3277621627834606517208177071759088097855048183641615082769528872043050020787, 19087113294497892618475669593723876605785307026981218038380435259594863105240],  // round 1
    [17645770319151120318035258350885823104235488352935695302274836429012504407725, 17990728141399065004015538797609951295983853332644474801890158217822768128628, 12607949331462269429981198199999740921418125994747028428126661151190418292729], // round 2
    [10025233623562179533044093426455032352895184661359005809314430689113735312874, 20398677688057466110325934731430812468657996794663167456321709689030080949228, 1944662263588038198375346521900053780907777056656211622999059135594196413076],  // round 3
    [12995068374816903282074967132431954020410301768622808407703775963080983755183, 0, 0],  // round 4
    [13278128079226679628648689279705910775020794457648431336050464485837924986341, 0, 0],  // round 5
    [21081768833381902942114733002158882075348844281359283013642620389621494952015, 0, 0],  // round 6
    [20751788049060260683191405008569080723662271828149227137187075968560831545739, 0, 0],  // round 7
    [20820291785607398388900832350860967875629907105847554413318238165275470374689, 0, 0],  // round 8
    [6971878585215744613467847324629115462668098071102846520957717612260531709386, 0, 0],   // round 9
    [21120353743307986506720883740380468652053382764895882204680310593048134053982, 0, 0],  // round 10
    [7853308243263055176258751393326645428041138029306706980470113526802326214700, 0, 0],   // round 11
    [17545076036297840030021082424260289805456380863517895917265467158332801090765, 0, 0],  // round 12
    [10740853637774754893036062076749871837371049036966225040269105665447180116170, 0, 0],  // round 13
    [24290796201833228559129233924595614281891670608675107544294264860003803501509, 0, 0],  // round 14
    [26722678647461522072509896114724736555938247563993442152746954157222882824350, 0, 0],  // round 15
    [20252491387019425681551488261397157776479297799360691728406809731508542196845, 0, 0],  // round 16
    [17070806525931584028449131949070191143344166668070820337429561524629464200550, 0, 0],  // round 17
    [25856554324149146992239414502939942208580094928192925471532421030223074525051, 0, 0],  // round 18
    [17714998974036855356530338446243137421735047395517260588250413348153258772076, 0, 0],  // round 19
    [20515196301761603016197694845695272699608637099106794944737311528118558777570, 0, 0],  // round 20
    [10100400556460905874275078234698187530913105549037797180493988678937053918124, 0, 0],  // round 21
    [12242010394227909997626655999345208835040087302065045201635069094289920778463, 0, 0],  // round 22
    [6838505804652359252670794375725267665530548946030641535297433541475260948424, 0, 0],   // round 23
    [21345718918993308853491352363460625447157796362108157527364130872100101143328, 0, 0],  // round 24
    [26397988737034501095129796920971941795766209722106383463197090306632188634870, 0, 0],  // round 25
    [27893799443241349360688137159923920340185830261519093384488134540544971987330, 0, 0],  // round 26
    [3102550735908358465878301372253437950829524988677083749179431098369388780259, 0, 0],   // round 27
    [2963742902601529003553690631564645593518709846059084207036841793643477514707, 0, 0],   // round 28
    [24620569969402072776192280888011017497854992833864712509770555543278833718751, 0, 0],  // round 29
    [25964807298150242099204032696543021731332498792173212422070959505270506288817, 0, 0],  // round 30
    [15107529391758643095716794813038523751713309080738989300826699946985294497278, 0, 0],  // round 31
    [26149402682269665088314773514719203730233986608723938665192802061570851149320, 0, 0],  // round 32
    [14300403008645647974330112479193012555289445502185868105642182233848475582899, 0, 0],  // round 33
    [1115361296285111421659408034287929280905078990986385263729179376131648187058, 0, 0],   // round 34
    [13081790983218231663826423630402269594642175266089309953018053418396572757728, 0, 0],  // round 35
    [8235521536407760690987948268259353704300918036393867110229857008864492272243, 0, 0],   // round 36
    [10466479494603471110085160358255184712338985686117376680963274257033378093044, 0, 0],  // round 37
    [10505351732961945434077967966272614185370876266035423475161721043839572600354, 0, 0],  // round 38
    [20492577817846125120765219135044390230365666103475157006227551523345028416653, 0, 0],  // round 39
    [9609702284002210167411637400029381999579573316818014884056109946803635903949, 0, 0],   // round 40
    [5550990570115355104018261990072269149174220738166262960442108003631983239538, 0, 0],   // round 41
    [4918607047827293284267178559571975167840449247468221935183514469924645319431, 0, 0],   // round 42
    [22327941647779098096798004328483144118875590152725522668881024275272944414051, 0, 0],  // round 43
    [12446460574596706595202266827006842340757403121130616325345603812748836460769, 0, 0],  // round 44
    [27189681619715898792962291676467917480208426216006102231934586868572227499788, 0, 0],  // round 45
    [8764133057432414528430129363242868018774698311348571048821261111352103735418, 0, 0],   // round 46
    [10306763858151643521688107379000096066251452823515924808024537509180120590301, 0, 0],  // round 47
    [18225419295569955709959264540894574108104760504259646575014370705413341659332, 0, 0],  // round 48
    [5586023775523332359482150390241085503538343981397337410273960103664896061318, 0, 0],   // round 49
    [1695250059692506203013076949309928562723368039356271828712091742435374824213, 0, 0],   // round 50
    [22405375952478438071934186194392756316305143676541092887399118393981076553314, 0, 0],  // round 51
    [10458537515857632902862111990551662021418451863412906712791662010765438376282, 0, 0],  // round 52
    [8572903186653093823393996934308987796939174550688349948095623496677370491349, 0, 0],   // round 53
    [26376584034857786080333290889331925169513316008268823389497001028535947725689, 0, 0],  // round 54
    [7665731866090251989409614325607843738907805016631894070026948889862506085680, 0, 0],   // round 55
    [4477111727391714901720242825845081209726923645743756019648580408082893195544, 0, 0],   // round 56
    [24052818921338006126265655523211029781246213697245853990174101542814404796394, 0, 0],  // round 57
    [2514052438055955996166191181555087722391472372457485613396039637466284876008, 0, 0],   // round 58
    [15915052994762066788134349126706966018769870301280961502696575741203715471620, 0, 0],  // round 59
    [14523364456910312064741833824384915970721514893070438053344001112980722532883, 8803956670977498145356652907229121212730957151393430875717273509992687004092, 10663098851045790689902302726099843869982323815663085170094254490745070606259],  // round 60
    [15771722055033274898971962892589697054998768026073330065443825965063277326619, 24009394270524302139600659893428847877911428012188014930479974124593827497896, 22073551841352628264163147373911841152201793863183403625155779300264423096775], // round 61
    [16843535002948632653135556540328830165745557071105115915108598045919908377862, 1518205506066737062294516413663386248913537376120439498858484657705789460110, 16130557973030629467749350011718803160555727145300402111387384840879624701824],  // round 62
    [8889838242573900603893251082243963471048473250580324046918980785903088175746, 26228644761030089864593236267771722990412818339075926138914275040572783608441, 12067734632794838098229971919863170976260163840996333398973186642649256640429],  // round 63
];


// chip configuration for Poseidon2 sponge and permutation
#[derive(Clone, Debug)]
pub struct Poseidon2ChipConfig {
    advice: [Column<Advice>; 31],   // 31 advice columns for a wide construction
    instance: Column<Instance>,     // for exposing public values
    s_init: Selector,               // ensures correct starting values at start of permutation
    s_e1: Selector,                 // selector computes RF/2 external rounds in one row (row 0)
    s_i: [Selector; 4],             // each selector computes 14 RP internal rounds giving us 14*4 = 56 internal rounds in 4 rows 
    s_e2: Selector,                 // selector computes the last RF/2 external rounds in one row (row 5)
    s_digest: Selector              // 
}


// structure for the Poseidon2 chip that holds the configuration
#[derive(Clone, Debug)]
pub struct Poseidon2Chip<F: PrimeField> {
    config: Poseidon2ChipConfig,
    _marker: PhantomData<F>
}


// implement the Chip trait for the PoseidonChip
impl<F: PrimeField> Chip<F> for Poseidon2Chip<F> {
    type Config = Poseidon2ChipConfig;
    type Loaded = ();

    // getter for the configuration
    fn config(&self) -> &Self::Config {
        &self.config
    }

    // getter for the loaded field
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}


// helper function to query an advice column at a rotation
fn adv<F: PrimeField>(meta: &mut VirtualCells<F>, col: Column<Advice>, rot: Rotation)
    -> Expression<F> { meta.query_advice(col, rot) }


// helper function for MixLayer (ML) using the circular matrix formed by (2, 1, 1) and the state vector x
fn mul_me<F: PrimeField>(x: &[Expression<F>; 3]) -> [Expression<F>; 3] {
    let s = x[0].clone() + x[1].clone() + x[2].clone();
    [x[0].clone() + s.clone(), x[1].clone() + s.clone(), x[2].clone() + s]
}


// helper fuction of MixLayer (ML) using the circuit matrix for internal rounds
fn mul_mi<F: PrimeField>(x: &[Expression<F>; 3]) -> [Expression<F>; 3] {
    let s = x[0].clone() + x[1].clone() + x[2].clone();
    [x[0].clone() + s.clone(), x[1].clone() + s.clone(), x[2].clone() + x[2].clone() + s.clone()]
}


// helper function to recompute lane values given starting lanes, and b cells (from S-Box outputs)
fn recompute_end_lanes<F: PrimeField>(
    meta: &mut VirtualCells<F>, 
    advice: [Column<Advice>; 31], 
    rot: Rotation
) -> [Expression<F>; 3] {
    let mut lanes = [adv(meta, advice[0], rot), adv(meta, advice[1], rot), adv(meta, advice[2], rot)];
    for i in 0..14 {
        let b = adv(meta, advice[4 + 2*i], rot);
        lanes = mul_mi(&[b, lanes[1].clone(), lanes[2].clone()]);
    }
    lanes
}


/*
* Gate Purpose: bake in the initial absorb for the t lanes to values [0, 0, IV] and constrain the values
* Columns used: adv0 -> lane0, adv1 -> lane1, adv2 -> lane2
* Constraints: [adv0_curr - 0, adv1_curr - 0, adv2_curr - IV]
*/
fn create_init_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 3], // pass A19, A21, A23 here 
    s_init: Selector,
    iv: F
) {
    meta.create_gate("ps_init", |meta| {
        let s_init = meta.query_selector(s_init);
        let b_prev = [
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];
        let lanes = mul_me(&b_prev);

        vec![
            s_init.clone() * lanes[0].clone(), 
            s_init.clone() * lanes[1].clone(), 
            s_init.clone() * (lanes[2].clone() - Expression::Constant(iv))
        ]
    });
}


/*
* Gate Purpose: compute RF/2 external rounds in the using the e1 selector
* Columns used: A0, ..., A23 for the 24 cells of SB q, b values, A24 and A25 for the elements to absorb
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_e1_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 31],
    s_e1: Selector,
    rc: [&[F]; 4] // pass the round constants needed (3 per round, 4 rounds total)
) {
    meta.create_gate("e1_gate", |meta| {
        let s_e1 = meta.query_selector(s_e1);
        let b_prev = [
            adv(meta, advice[19], Rotation::prev()), 
            adv(meta, advice[21], Rotation::prev()), 
            adv(meta, advice[23], Rotation::prev())
        ];
        let m0 = adv(meta, advice[24], Rotation::cur()); // this rows messsage elements to be absorbed 
        let m1 = adv(meta, advice[25], Rotation::cur());
        let mut state = mul_me(&b_prev); // this is the ML layer for the end of the last permutation
        state[0] = state[0].clone() + m0; // absorb the next block now that the last permutation is finished
        state[1] = state[1].clone() + m1; 
        let mut lanes = mul_me(&state); // initial ML transformation before any external round happens
        
        let mut constraints = vec![]; // vector to hold all constraints
        for i in 0..4 { // for loop with 4 iterations since we are doing RF / 2 = 4 rounds in this gate
            let mut sbox_out = vec![]; // vector to hold the S-Box outputs
            for j in 0..3 { // perform ARC and SB over all 3 lanes, storing S-Box output in sbox_out
                let a = lanes[j].clone() + Expression::Constant(rc[i][j]); 
                let a_sq = adv(meta, advice[6*i + 2*j], Rotation::cur()); // multiply by 6 to get to the correct start col, each RF round takes 6 cols
                let b = adv(meta, advice[6*i + 2*j + 1], Rotation::cur()); // a_sq = a*a, b = a_sq * a_sq * a = a^5
                sbox_out.push(b.clone());

                // 24 constraints over the 12 a_sq values and the 12 b values
                constraints.push(s_e1.clone() * (a_sq.clone() - a.clone() * a.clone()));
                constraints.push(s_e1.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));
            }
            if i < 3 { lanes = mul_me(&[sbox_out[0].clone(), sbox_out[1].clone(), sbox_out[2].clone()]); }
        }

        constraints // return the constraints as required by create_gate
    });
}


/*
* Gate Purpose: compute RP/4 = 14 internal rounds in the next row using the i0 selector
* Columns used: A0, A1, A2 for the current state (ML from previous row), q and b in the subsequent pairs of cols
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_i0_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 31],
    s_i0: Selector,
    rc: &[F] // 14 RP rounds, each RP round only has lane0 go into ARC
) {
    meta.create_gate("i0_gate", |meta| {
        let s_i0 = meta.query_selector(s_i0);
        let b_prev = [
            adv(meta, advice[19], Rotation::prev()), 
            adv(meta, advice[21], Rotation::prev()), 
            adv(meta, advice[23], Rotation::prev())
        ];
        let start = mul_me(&b_prev); // complete last ML which was deferred from E1 to here
        let mut lanes_cur = [ // A0, A1, A2 hold MDS(A19_prev, A21_prev, A23_prev)
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];

        // constraints for the I0 gate
        let mut constraints = vec![];
        for i in 0..3 { // 3 constraints for A0, A1, A2 <-> A19, A21, A23 (previous row, MDS multiplied)
            constraints.push(s_i0.clone() * (lanes_cur[i].clone() - start[i].clone()));
        }

        // 14 internal RP rounds 
        for i in 0..14 {
            let a = lanes_cur[0].clone() + Expression::Constant(rc[i]); // ARC
            let a_sq = adv(meta, advice[3 + 2*i], Rotation::cur()); // a_sq stored in A3, b stored in A4 and so on
            let b = adv(meta, advice[3 + 2*i + 1], Rotation::cur());

            // 28 constraints, 2 per RP round performed
            constraints.push(s_i0.clone() * (a_sq.clone() - a.clone() * a.clone()));
            constraints.push(s_i0.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));

            if i < 13 { lanes_cur = mul_mi(&[b, lanes_cur[1].clone(), lanes_cur[2].clone()]); } // deferred to I1 for last I0 RP round
        }
        constraints
    });
}


/*
* Gate Purpose: compute RP/4 = 14 internal rounds in the next row using the int selector
* This helper function creates all 3 internal gates but needs to be 3 distinct gates because the round constants are different
* Columns used: A0, A1, A2 for the state after ML from previous row, q and b in the subsequent pairs of cols
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_int_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 31],
    s_int: Selector,
    rc: &[F]
) {
    meta.create_gate("int_gate", |meta| {
        let s_int = meta.query_selector(s_int);
        let start = recompute_end_lanes(meta, advice, Rotation::prev());
        let mut lanes_cur = [ // A0, A1, A2 hold MDS(A19_prev, A21_prev, A23_prev)
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];

        // constraints for the internal gate(s)
        let mut constraints = vec![];
        for i in 0..3 { // 3 constraints for A0, A1, A2 <-> reconstruct_end_lanes()
            constraints.push(s_int.clone() * (lanes_cur[i].clone() - start[i].clone()));
        }

        // 14 internal RP rounds 
        for i in 0..14 {
            let a = lanes_cur[0].clone() + Expression::Constant(rc[i]); // ARC
            let a_sq = adv(meta, advice[3 + 2*i], Rotation::cur()); // a_sq stored in A3, b stored in A4 and so on
            let b = adv(meta, advice[3 + 2*i + 1], Rotation::cur());

            // 28 constraints, 2 per RP round performed
            constraints.push(s_int.clone() * (a_sq.clone() - a.clone() * a.clone()));
            constraints.push(s_int.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));

            if i < 13 { lanes_cur = mul_mi(&[b, lanes_cur[1].clone(), lanes_cur[2].clone()]); } // deferred to next gate for last RP round
        }
        constraints
    });
}


/*
* Gate Purpose: compute RF/2 external rounds in the using the e2 selector
* Columns used: A0, ..., A31: (A0, A1, A2) are the MDS outputs but the lanes need to be recomputed
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_e2_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 31],
    s_e2: Selector,
    rc: [&[F]; 4] // pass the round constants needed (3 per round, 4 rounds total)
) {
    meta.create_gate("e2_gate", |meta| {
        let s_e2 = meta.query_selector(s_e2);
        let mut lanes = recompute_end_lanes(meta, advice, Rotation::prev()); // I3 end lane values (after ML)
        let mut constraints = vec![];

        // perform and constrain q, b over 4 RF rounds
        for i in 0..4 {
            let mut sbox_out = vec![];
            for j in 0..3 {
                let a = lanes[j].clone() + Expression::Constant(rc[i][j]); 
                let a_sq = adv(meta, advice[6*i + 2*j], Rotation::cur()); // multiply by 6 to get to the correct start col, each RF round takes 6 cols
                let b = adv(meta, advice[6*i + 2*j + 1], Rotation::cur()); // a_sq = a*a, b = a_sq * a_sq * a = a^5
                sbox_out.push(b.clone());

                // 24 constraints over the 12 a_sq values and the 12 b values
                constraints.push(s_e2.clone() * (a_sq.clone() - a.clone() * a.clone()));
                constraints.push(s_e2.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));
            }
            if i < 3 { lanes = mul_me(&[sbox_out[0].clone(), sbox_out[1].clone(), sbox_out[2].clone()]); }
        }
        constraints
    });
}


/*
* Gate Purpose: maps the digest from the end of the permutation to the instance column
* Columns used: instance, SB outputs from E2 (A19, A21, A23)
* Constraints: ensure equality between the hash in the instance column and the squeezed element
*/
fn create_digest_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4], // pass A19, A21, A23 here
    instance: Column<Instance>, 
    s_digest: Selector
) {
    meta.create_gate("digest_gate", |meta| {
        let s_digest = meta.query_selector(s_digest);
        let b_prev = [
            adv(meta, advice[0], Rotation::cur()), 
            adv(meta, advice[1], Rotation::cur()), 
            adv(meta, advice[2], Rotation::cur())
        ];
        let mut lanes = mul_me(&b_prev); // E2 end lane values (after ML)
        let hash = adv(meta, advice[3], Rotation::cur()); // gets the hash value written into the instance column

        // only constraint is ensuring equality between the rate element squeezed and the hash
        vec![
            s_digest * (lanes[0].clone() - hash)
        ]
    });
}


// implementation of additional methods for the Poseidon2 chip
impl<F: PrimeField> Poseidon2Chip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Poseidon2Chip { config, _marker: PhantomData }
    }

    // configure the chip with all the gates and constraints
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 31],
        instance: Column<Instance>,
        iv: F,
        rc: &[F; 80] // 24 RCs for RF rounds, 56 RCs for RP rounds, 80 total
    ) -> <Self as Chip<F>>::Config {
        // enable equality on instance column and A24 to copy constrain digest values
        meta.enable_equality(advice[24]);
        meta.enable_equality(advice[25]);
        meta.enable_equality(instance);

        // make the needed selectors
        let s_init = meta.selector();
        let s_digest = meta.selector();
        let s_e1 = meta.selector();
        let s_e2 = meta.selector();
        let s_i0 = meta.selector();
        let s_i1 = meta.selector();
        let s_i2 = meta.selector();
        let s_i3 = meta.selector();

        create_init_gate(meta, [advice[19], advice[21], advice[23]], s_init, iv);
        // create_e1_gate(meta, advice, s_e1, [&rc[0..3], &rc[3..6], &rc[6..9], &rc[9..12]]);
        // create_i0_gate(meta, advice, s_i0, &rc[24..38]);
        // create_int_gate(meta, advice, s_i1, &rc[38..52]);
        // create_int_gate(meta, advice, s_i2, &rc[52..66]);
        // create_int_gate(meta, advice, s_i3, &rc[66..80]);
        // create_e2_gate(meta, advice, s_e2, [&rc[12..15], &rc[15..74], &rc[74..77], &rc[77..80]]);
        create_digest_gate(meta, [advice[19], advice[21], advice[23], advice[24]], instance, s_digest);

        Poseidon2ChipConfig {
            advice,
            instance, 
            s_init,
            s_e1,
            s_i: [s_i0, s_i1, s_i2, s_i3],
            s_e2,
            s_digest
        }
    }
}


// trait for the sub-functions of the poseidon permutation
pub trait Poseidon2Instructions<F: PrimeField>: Chip<F> {
    type Num;

    // permutation
    fn permute(
        &self, 
        layouter: &mut impl Layouter<F>,
        state: [AssignedCell<F, F>; 4]
    ) -> Result<[AssignedCell<F, F>; 4], Error>;
}


// implement the needed functionality for synthesis with the chip
impl<F: PrimeField> Poseidon2Instructions<F> for Poseidon2Chip<F> {
    type Num = Number<F>;

    // permutation function
    fn permute(
        &self, 
        mut layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 3]
    ) -> Result<[AssignedCell<F, F>; 4], Error> {
        let chip_config = self.config();
    }
} 