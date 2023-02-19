use crate::rewriting_utilities::SemiCompressedTermTree::{Compressed, Variable, Explicit};
use std::collections::{VecDeque, HashSet};
use ahash::AHashMap as HashMap;
use term_pool::{Symbol, TermPool, StoredTerm, Term0, Term1, Term2, Term3, Term4, Term5, Term6, Term7Plus};
use term_pool::position::ExplicitPosition;
use crate::set_automaton::State;
use crate::InnermostRewriter;

/// A SemiCompressedTermTree (SCTT) is a mix between a StoredTerm and a TermSyntaxTree and is used
/// to represent the rhs of rewrite rules and the lhs and rhs of conditions.
///
/// It stores as much as possible in the term pool. Due to variables it cannot be fully compressed.
/// For variables it stores the position in the lhs of a rewrite rule where the concrete term can be
/// found that will replace the variable.
///
/// # Examples
/// For the rewrite rule and(true, true) = true, the SCTT of the rhs will be of type Compressed, with
/// a pointer to the term true.
///
/// For the rewrite rule minus(x, 0) = x, the SCTT of the rhs will be of type Variable, storing position
/// 1, the position of x in the lhs.
///
/// For the rewrite rule minus(s(x), s(y)) = minus(x, y), the SCTT of the rhs will be of type
/// Explicit, which will stored the head symbol 'minus' and two child SCTTs of type Variable.
#[derive(Debug,Clone,PartialEq,Eq,Hash,Ord, PartialOrd)]
pub(crate) enum SemiCompressedTermTree {
    Explicit(ExplicitNode),
    Compressed(StoredTerm),
    Variable(ExplicitPosition)
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,Ord, PartialOrd)]
pub(crate) struct ExplicitNode {
    pub(crate) head: Symbol,
    pub(crate) children: Vec<SemiCompressedTermTree>
}

impl SemiCompressedTermTree {
    fn is_compressed(&self) -> bool {
        if let Compressed(_) = self {
            true
        } else {
            false
        }
    }

    /// Given a StoredTerm lhs and a term pool this function instantiates the SCTT and computes a StoredTerm.
    ///
    /// # Example
    /// For the SCTT belonging to the rewrite rule minus(s(x), s(y)) = minus(x, y)
    /// and the concrete lhs minus(s(0), s(0)) the evaluation will go as follows.
    /// evaluate will encounter an ExplicitNode and make two recursive calls to get the subterms.
    /// Both these recursive calls will return the term '0'.
    /// The term pool will be used to construct the term minus(0, 0).
    pub(crate) fn evaluate(&self, lhs: &StoredTerm, tp: &mut TermPool) -> StoredTerm {
        match self {
            Explicit(en) => {
                let symbol_index= en.head;
                let len = en.children.len();
                match len {
                    0 => {
                        let new_term = Term0{ symbol_index };
                        tp.construct_or_find_term_0(new_term)
                    }
                    1 => {
                        let new_term = Term1{ symbol_index, subterms: [en.children[0].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_1(new_term)
                    }
                    2 => {
                        let new_term = Term2{ symbol_index, subterms:  [en.children[0].evaluate(lhs,tp),en.children[1].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_2(new_term)
                    }
                    3 => {
                        let new_term = Term3{ symbol_index, subterms:  [en.children[0].evaluate(lhs,tp),en.children[1].evaluate(lhs,tp),en.children[2].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_3(new_term)
                    }
                    4 => {
                        let new_term = Term4{ symbol_index, subterms:  [en.children[0].evaluate(lhs,tp),en.children[1].evaluate(lhs,tp),en.children[2].evaluate(lhs,tp),en.children[3].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_4(new_term)
                    }
                    5 => {
                        let new_term = Term5{ symbol_index, subterms:  [en.children[0].evaluate(lhs,tp),en.children[1].evaluate(lhs,tp),en.children[2].evaluate(lhs,tp),en.children[3].evaluate(lhs,tp),en.children[4].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_5(new_term)
                    }
                    6 => {
                        let new_term = Term6{ symbol_index, subterms:  [en.children[0].evaluate(lhs,tp),en.children[1].evaluate(lhs,tp),en.children[2].evaluate(lhs,tp),en.children[3].evaluate(lhs,tp),en.children[4].evaluate(lhs,tp),en.children[5].evaluate(lhs,tp)] };
                        tp.construct_or_find_term_6(new_term)
                    }
                    _ => {
                        let mut subterms = Vec::with_capacity(len);
                        for i in 0..len {
                            subterms.push(en.children[i].evaluate(lhs,tp));
                        }
                        let new_term = Term7Plus{ symbol_index, subterms };
                        tp.construct_or_find_term_7_plus(new_term)
                    }
                }
            }
            Compressed(ct) => {ct.clone()}
            Variable(p) => {lhs.get_position(&p).clone()}
        }
    }

    /// An alternative version of evaluate that accepts the lhs in parts:
    /// the head symbol and a slice of subterms.
    pub(crate) fn evaluate_alt(&self, lhs_head: Symbol, lhs_subterms: &[StoredTerm], tp: &mut TermPool) -> StoredTerm {
        match self {
            Explicit(en) => {
                let symbol_index= en.head;
                let len = en.children.len();
                match len {
                    0 => {
                        let new_term = Term0{ symbol_index };
                        tp.construct_or_find_term_0(new_term)
                    }
                    1 => {
                        let new_term = Term1{ symbol_index, subterms: [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_1(new_term)
                    }
                    2 => {
                        let new_term = Term2{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_2(new_term)
                    }
                    3 => {
                        let new_term = Term3{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_3(new_term)
                    }
                    4 => {
                        let new_term = Term4{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_4(new_term)
                    }
                    5 => {
                        let new_term = Term5{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[4].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_5(new_term)
                    }
                    6 => {
                        let new_term = Term6{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[4].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[5].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        tp.construct_or_find_term_6(new_term)
                    }
                    _ => {
                        let mut subterms = Vec::with_capacity(len);
                        for i in 0..len {
                            subterms.push(en.children[i].evaluate_alt(lhs_head, lhs_subterms,tp));
                        }
                        let new_term = Term7Plus{ symbol_index, subterms };
                        tp.construct_or_find_term_7_plus(new_term)
                    }
                }
            }
            Compressed(ct) => {ct.clone()}
            Variable(p) => {
                lhs_subterms[p.indices[0] - 1].get_position_indices(&p.indices[1..]).clone()
            }
        }
    }

    /// Used by the innermost rewriter. At the top level it does not construct a StoredTerm but keeps
    /// the term in parts: the head symbol and the subterms. It invokes the InnermostRewriter on the
    /// results of the SCTT evaluation. The head symbol and the subterms are seperated for performance
    /// reasons; it is better to avoid calls to the term pool. See docs InnermostRewriter.
    #[inline(always)]
    pub(crate) fn evaluate_and_rewrite(&self, states: &Vec<State>, tp: &mut TermPool, lhs_head: Symbol, lhs_subterms: &[StoredTerm], nf_subterms: bool) -> StoredTerm {
        match self {
            Explicit(en) => {
                let symbol_index= en.head;
                let len = en.children.len();
                match len {
                    0 => {
                        let new_term = Term0{ symbol_index };
                        InnermostRewriter::rewrite_aux_0(states,tp,new_term)
                    }
                    1 => {
                        let new_term = Term1{ symbol_index, subterms: [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_1(states,tp,new_term, nf_subterms)
                    }
                    2 => {
                        let new_term = Term2{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_2(states,tp,new_term, nf_subterms)
                    }
                    3 => {
                        let new_term = Term3{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_3(states,tp,new_term, nf_subterms)
                    }
                    4 => {
                        let new_term = Term4{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_4(states,tp,new_term, nf_subterms)
                    }
                    5 => {
                        let new_term = Term5{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[4].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_5(states,tp,new_term, nf_subterms)
                    }
                    6 => {
                        let new_term = Term6{ symbol_index, subterms:  [en.children[0].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[1].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[2].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[3].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[4].evaluate_alt(lhs_head, lhs_subterms,tp),en.children[5].evaluate_alt(lhs_head, lhs_subterms,tp)] };
                        InnermostRewriter::rewrite_aux_6(states,tp,new_term, nf_subterms)
                    }
                    _ => {
                        let mut subterms = Vec::with_capacity(len);
                        for i in 0..len {
                            subterms.push(en.children[i].evaluate_alt(lhs_head, lhs_subterms,tp));
                        }
                        let new_term = Term7Plus{ symbol_index, subterms };
                        InnermostRewriter::rewrite_aux_7_plus(states,tp,new_term, nf_subterms)
                    }
                }
            }
            Compressed(st) => {InnermostRewriter::rewrite_aux(states,tp,st.clone(), &mut None)}
            Variable(p) => {
                let subterm = lhs_subterms[p.indices[0] - 1].get_position_indices(&p.indices[1..]).clone();
                InnermostRewriter::rewrite_aux(states,tp,subterm, &mut None)
            }
        }
    }

    /// Creates a SCTT from a term. The var_map parameter should specify where which variable can be
    /// found in the lhs of the rewrite rule.
    pub(crate) fn from_term(t: StoredTerm, tp: &TermPool, var_map: &HashMap<Symbol,ExplicitPosition>) -> SemiCompressedTermTree {
        if t.get_subterms().len() == 0 {
            if var_map.contains_key(&t.get_head_symbol()) {
                Variable(var_map.get(&t.get_head_symbol()).unwrap().clone())
            } else {
                Compressed(t.clone())
            }
        } else {
            let children = t.get_subterms().iter().map(|c| SemiCompressedTermTree::from_term(c.clone(),tp,var_map)).collect();
            let en = ExplicitNode{ head: t.get_head_symbol(), children };
            if en.children.iter().all(|c| c.is_compressed()) {
                Compressed(t.clone())
            } else {
                Explicit(en)
            }
        }
    }

    /// Used to check if a subterm is duplicated, for example in times(s(x), y) = plus(y, times(x,y))
    pub(crate) fn contains_duplicate_var_references(&self) -> bool {
        let references = self.get_all_var_references();
        let mut seen = HashSet::new();
        for r in references {
            if seen.contains(&r) {
                return true;
            }
            seen.insert(r);
        }
        false
    }

    /// If the subterms are in normal form this boolean indicate whether the rewrite rules preserves
    /// that. For example, for the rewrite rule minus(s(x), s(y)) = minus(x, y) nf_subterms return true.
    /// This is used in an innermost rewriting strategy.
    pub(crate) fn nf_subterms(&self) -> bool {
        match self {
            Explicit(en) => {
                let mut result = true;
                for c in &en.children {
                    result = result && c.nf_subterms_aux()
                }
                result
            }
            Compressed(_) => {false}
            Variable(_) => {true}
        }
    }

    fn nf_subterms_aux(&self) -> bool {
        match self {
            Explicit(_) => {false}
            Compressed(_) => {false}
            Variable(_) => {true}
        }
    }

    /// Get all references to variables, including duplicates
    fn get_all_var_references(&self) -> Vec<ExplicitPosition> {
        let mut result = vec![];
        match self {
            Explicit(en) => {
                for n in &en.children {
                    result.extend_from_slice(&n.get_all_var_references());
                }
            }
            Compressed(_) => {}
            Variable(ep) => {result.push(ep.clone());}
        }
        result
    }
}

/// Create a mapping of variables to their position in 't'
pub(crate) fn create_var_map(t: StoredTerm, arity_per_symbol: &HashMap<Symbol,usize>) -> HashMap<Symbol,ExplicitPosition> {
    //Queue of pairs of subterm and position in term t that need to be inspected
    let mut queue = VecDeque::new();
    queue.push_back((t,ExplicitPosition::empty_pos()));
    let mut map = HashMap::new();
    while !queue.is_empty() {
        //get a subterm to inspect
        let (term, pos) = queue.pop_front().unwrap();
        let head = term.get_head_symbol();
        //If it is not in the arity_per_symbol map it is a variable
        if !arity_per_symbol.contains_key(&head) {
            map.insert(head,pos.clone());
        }
        let mut i = 1;
        //Put subterms in the queue
        for sub in term.get_subterms() {
            let mut sub_pos = pos.clone();
            sub_pos.indices.push(i);
            queue.push_back((sub.clone(),sub_pos.clone()));
            i += 1;
        }
    }
    map
}

#[allow(unused_imports)]
mod tests {
    use term_pool::{TermPool, TermSyntaxTree};
    use ahash::AHashMap as HashMap;
    use crate::rewriting_utilities::{SemiCompressedTermTree, ExplicitNode};
    use crate::rewriting_utilities::SemiCompressedTermTree::{Compressed, Variable, Explicit};
    use term_pool::position::ExplicitPosition;
    use smallvec::smallvec;

    #[test]
    fn test_constant() {
        let mut tp = TermPool::new();
        let tst = TermSyntaxTree::from_string("a").unwrap();
        let t = tp.construct_term_from_scratch(tst);
        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(t.clone(), &tp, &map);
        assert_eq!(sctt,Compressed(t));
    }

    #[test]
    fn test_compressible() {
        let mut tp = TermPool::new();
        let tst = TermSyntaxTree::from_string("f(a,a)").unwrap();
        let t = tp.construct_term_from_scratch(tst);
        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(t.clone(), &tp, &map);
        assert_eq!(sctt,Compressed(t));
    }

    #[test]
    fn not_compressible() {
        let mut tp = TermPool::new();
        let tst = TermSyntaxTree::from_string("f(x,x)").unwrap();
        let t = tp.construct_term_from_scratch(tst);
        let mut map = HashMap::new();
        let var_pos = ExplicitPosition{ indices: smallvec![1] };
        map.insert(*tp.string_to_symbol_index.get("x").unwrap(),var_pos.clone());
        let sctt = SemiCompressedTermTree::from_term(t, &tp, &map);
        let en = Explicit(ExplicitNode{
            head: *tp.string_to_symbol_index.get("f").unwrap(),
            children: vec![
                Variable(var_pos.clone()),
                Variable(var_pos.clone())
            ] });
        assert_eq!(sctt,en);
    }

    #[test]
    fn partly_compressible() {
        let mut tp = TermPool::new();
        let tst = TermSyntaxTree::from_string("f(f(a,a),x)").unwrap();
        let t = tp.construct_term_from_scratch(tst);
        let tst_compressible = TermSyntaxTree::from_string("f(a,a)").unwrap();
        let compressible = tp.construct_term_from_scratch(tst_compressible);
        let mut map = HashMap::default();
        let var_pos = ExplicitPosition{ indices: smallvec![1] };
        map.insert(*tp.string_to_symbol_index.get("x").unwrap(),var_pos.clone());
        let sctt = SemiCompressedTermTree::from_term(t, &tp, &map);
        let en = Explicit(ExplicitNode{
            head: *tp.string_to_symbol_index.get("f").unwrap(),
            children: vec![
                Compressed(compressible),
                Variable(var_pos.clone())
            ] });
        assert_eq!(sctt,en);
    }

    #[test]
    fn test_evaluation() {
        let mut tp = TermPool::new();
        let tst_rhs = TermSyntaxTree::from_string("f(f(a,a),x)").unwrap();
        let t_rhs = tp.construct_term_from_scratch(tst_rhs);
        let tst_lhs = TermSyntaxTree::from_string("g(b)").unwrap();
        let t_lhs = tp.construct_term_from_scratch(tst_lhs);
        let mut map = HashMap::default();
        let var_pos = ExplicitPosition{ indices: smallvec![1] };
        map.insert(*tp.string_to_symbol_index.get("x").unwrap(),var_pos.clone());
        let sctt = SemiCompressedTermTree::from_term(t_rhs, &tp, &map);
        let tst_expected= TermSyntaxTree::from_string("f(f(a,a),b)").unwrap();
        let t_expected = tp.construct_term_from_scratch(tst_expected);
        assert_eq!(sctt.evaluate(&t_lhs,&mut tp), t_expected);
    }

}
