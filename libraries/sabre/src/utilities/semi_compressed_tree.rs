
use mcrl2_rust::atermpp::{Symbol, ATerm};
use crate::utilities::ExplicitPosition;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SemiCompressedTermTree 
{
    Explicit(ExplicitNode),
    Compressed(ATerm),
    Variable(ExplicitPosition)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ExplicitNode 
{
    pub head: Symbol,
    pub children: Vec<SemiCompressedTermTree>
}

/*
impl SemiCompressedTermTree {
    fn is_compressed(&self) -> bool 
    {
        if let Compressed(_) = self {
            true
        } else {
            false
        }
    }

    /// Given a ATerm left hand side and a term pool this function instantiates the SCTT and computes a StoredTerm.
    ///
    /// # Example
    /// For the SCTT belonging to the rewrite rule minus(s(x), s(y)) = minus(x, y)
    /// and the concrete lhs minus(s(0), s(0)) the evaluation will go as follows.
    /// evaluate will encounter an ExplicitNode and make two recursive calls to get the subterms.
    /// Both these recursive calls will return the term '0'.
    /// The term pool will be used to construct the term minus(0, 0).
    pub fn evaluate(&self, tp: &mut TermPool, lhs: &ATerm) -> ATerm 
    {
        match self {
            Explicit(en) => {                
                let mut subterms = Vec::with_capacity(len);
                for i in 0..len {
                    subterms.push(en.children[i].evaluate(lhs,tp));
                }
                let new_term = Term7Plus{ symbol_index, subterms };
                tp.create(new_term)
            }
            Compressed(ct) => { ct.clone() }
            Variable(p) => { get_position(&lhs, &p).clone() }
        }
    }

    /// Creates a SCTT from a term. The var_map parameter should specify where which variable can be
    /// found in the lhs of the rewrite rule.
    pub(crate) fn from_term(t: StoredTerm, tp: &TermPool, var_map: &HashMap<Symbol,ExplicitPosition>) -> SemiCompressedTermTree 
    {
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
    pub(crate) fn contains_duplicate_var_references(&self) -> bool 
    {
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
pub(crate) fn create_var_map(t: &ATerm) -> HashMap<Symbol,ExplicitPosition> 
{
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
*/