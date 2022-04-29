int search (int alpha, int beta, int move, node parent, int depth) {
    node current;
    int extend, fmax, fscore, tt_hit;
    /* declare the local variables that require constant initialization */
    int fprune = 0;
    int fpruned_moves = 0;
    int score = -infinite_val;
    /* execute the opponent's move and determine how to extend the search */
    make_move(parent, move, &current;
    extend = extensions(move, current, depth);
    depth += extend;
    /* decide about limited razoring at the pre-pre-frontier nodes */
    fscore = (mat_balance(current) + razor_margin);
    if (!extend && (depth == pre_pre_frontier) && (fscore <= alpha)
        { fprune = 1; score = fmax = fscore; }
    /* decide about extended futility pruning at pre-frontier nodes */
    fscore = (mat_balance(current) + extd_futil_margin);
    if (!extend && (depth == pre_frontier) && (fscore <= alpha))
        { fprune = 1; score = fmax = fscore; }
    /* decide about selective futility pruning at frontier nodes */
    fscore = (mat_balance(current) + futil_margin);
    if (!check(move) && (depth == frontier) && (fscore <= alpha))
        { fprune = 1; score = fmax = fscore; }
 
    /* ... */
 
    /* probe the transposition tables at the current node */
    tt_hit = probe_transposition_tables(current, depth, &tt_ref);
    if (tt_hit) {/* ... */} else {/* ... */}
    /* try the adaptive null-move search with a minimal window around */
    /* "beta" only if it is allowed, desired, and really promises to cut off */
    if (!fprune && !check(move) && null_okay(current, move)
          && try_null(alpha, beta, current, depth, move, tt_ref)) {
         /* ... */
         null_score = -search(-beta, -beta + 1, null_move, current,
                         depth - R_adpt(current, depth) - 1);
         if (null_score >= beta) return null_score;
         /* ... */              /*fail-high null-move cutoff*/
    }
 
    /* ... */
    /* now continue as usual but prune all futile moves if "fprune == 1"*/
    for (move = first(current), (move != 0), move = next(current, move))
        if (!fprune || check(move)                    /*recursive PVS part*/
                    || (fmax + mat_gain(move) > alpha)) {/* ... */}
        else fpruned_moves++;
    /* "fpruned_moves > 0" => the search was selective at the current node */
    /* ... */
 }