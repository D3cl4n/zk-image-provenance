from sage.all import GF


# class for computing the Poseidon2 hash
class Poseidon2:
    def __init__(self, p, security_level, alpha, r, t, rf, rp, mds_e, mds_i, rcs):
        self.p = p
        self.field_p = GF(p)
        self.security_level = security_level
        self.alpha = alpha
        self.r = r
        self.t = t
        self.rf = rf
        self.rp = rp
        self.mds_e = mds_e
        self.mds_i = mds_i
        self.rcs = [self.field_p(x) for x in rcs]
        self.state = [self.field_p(0)] * self.t # swap with iv?

    # pow5 helper function
    def pow5(self, x):
        return self.field_p(x**self.alpha)

    # sub bytes helper function for external rounds
    def sb_e(self):
        return [self.pow5(self.state[0], self.pow5(self.state[1], self.pow5(self.state[2])))]

    # sub bytes helper function for internal rounds
    def sb_i(self):
        return [self.pow5(self.state[0], self.state[1], self.state[2])]

    # add round constants helper function for external rounds
    def arc_e(self, constants):
        assert len(constants) == self.t

        return [self.field_p(self.state[0] + constants[0]), self.field_p(self.state[1] + constants[1]), self.field_p(self.state[2] + constants[2])]

    # add round constants helper function for internal rounds
    def arc_i(self, constant):
        return [self.field_p(self.state[0] + constant), self.state[1], self.state[2]]

    # generic MDS multiplication - pass mds_i or mds_e depending on round
    def mds_mul(self, mds):
        next_state = []
        next_state.append(self.field_p(self.state[0] * mds[0][0] + self.state[1] * mds[0][1] + self.state[2] * mds[0][2] + self.state[3] * mds[0][3]))
        next_state.append(self.field_p(self.state[0] * mds[1][0] + self.state[1] * mds[1][1] + self.state[2] * mds[1][2] + self.state[3] * mds[1][3]))
        next_state.append(self.field_p(self.state[0] * mds[2][0] + self.state[1] * mds[2][1] + self.state[2] * mds[2][2] + self.state[3] * mds[2][3]))
        next_state.append(self.field_p(self.state[0] * mds[3][0] + self.state[1] * mds[3][1] + self.state[2] * mds[3][2] + self.state[3] * mds[3][3]))

        return next_state

    # Poseidon2 permutation
    def permute(self):
        rc_idx = 0
        for _ in range(0, self.rf // 2):
            pass

        for _ in range(0, self.rp):
            pass

        for _ in range(0, self.rf // 2):
            pass

    # absorb the input
    def absorb(self, inputs):
        assert len(inputs) == self.r
        self.state = [self.field_p(self.state[0] + inputs[0]), self.field_p(self.state[1] + inputs[1]), self.state[2]]
    
    # squeeze state[1] as output
    def squeeze(self):
        return self.state[1]
        
    # hash given a padded and packed preimage
    def hash(self, preimage_blocks):
        for block in preimage_blocks:
            self.absorb(block)
            self.permute()
    
        return self.squeeze()