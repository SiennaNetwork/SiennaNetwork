module.exports = function row2record (row) {
  const [
    _A_, _B_, _C_, _D_, _E_, _F_, _G_, _H_, _I_, _J_, _K_, _L_,
    _M_, _N_, _O_, _P_, _Q_, _R_, _S_, _T_, _U_, _V_, _W_
  ] = row
  const data = {
    total:            Sienna  (_A_),      // Schedule

    pool:             String  (_B_||''),  // Pool
    subtotal:         Sienna  (_C_),

    name:             String  (_D_||''),  // Account
    amount:           Sienna  (_E_),
    percent_of_total: Percent (_F_),
    address:          String  (_G_),

    start_at_days:    Days    (_H_, _I_), // Account timing
    start_at:         Seconds (_I_),
    cliff_percent:    Percent (_J_),
    cliff:            Sienna  (_K_),
    interval_days:    Days    (_L_, _M_),
    interval:         Seconds (_M_) || 0,
    portions:         Number  (_N_),
    portion_size:     Sienna  (_O_),
    duration_days:    Days    (_P_, _Q_),
    duration:         Seconds (_Q_) || 0,
    remainder:        Sienna  (_R_),
  }
  return data
}

const assert = (...args) => {
  require('assert')(...args)
  return true // to use in && chains
}

const ONE_SIENNA        = BigInt('1000000000000000000')
const THOUSANDTH_SIENNA = BigInt(   '1000000000000000')

const isN = x => !isNaN(x)

const Sienna = x => isN(x) && BigInt(x*1000) * THOUSANDTH_SIENNA

const Percent = x => x && x/100

const Days = (x, y) =>
  x && isN(x) && y && isN(y) &&
  assert(x*24*60*60 === y, `${x} days must be accompanied with ${y} seconds`) &&
  x

const Seconds = x => x

const Address = x =>
  x && assert(x.startsWith('secret1')) &&
  assert(x.length === 45, `address must be 45 characters: ${x}`) &&
  x
