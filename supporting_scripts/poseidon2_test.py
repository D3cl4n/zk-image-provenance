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