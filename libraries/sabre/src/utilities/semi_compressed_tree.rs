
use std::collections::{HashMap, VecDeque, HashSet};

use mcrl2_rust::atermpp::{Symbol, ATerm, TermPool};
use crate::utilities::ExplicitPosition;

/// A SemiCompressedTermTree (SCTT) is a mix between a [ATerm] and a TermSyntaxTree and is used
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

/// St
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ExplicitNode 
{
    pub head: Symbol,
    pub children: Vec<SemiCompressedTermTree>
}

use SemiCompressedTermTree::*;

use super::get_position;

impl SemiCompressedTermTree 
{
    fn is_compressed(&self) -> bool 
    {
        if let Compressed(_) = self {
            true
        } else {
            false
        }
    }

    /// Given a [ATerm] left hand side and a term pool this function instantiates the SCTT and computes a [ATerm].
    ///
    /// # Example
    /// For the SCTT belonging to the rewrite rule minus(s(x), s(y)) = minus(x, y)
    /// and the concrete lhs minus(s(0), s(0)) the evaluation will go as follows.
    /// evaluate will encounter an ExplicitNode and make two recursive calls to get the subterms.
    /// Both these recursive calls will return the term '0'.
    /// The term pool will be used to construct the term minus(0, 0).
    pub fn evaluate(&self, lhs: &ATerm, tp: &mut TermPool) -> ATerm 
    {
        match self {
            Explicit(node) => {  
                // Create an ATerm with as arguments all the evaluated semi compressed term trees.              
                let mut subterms = Vec::with_capacity(node.children.len());

                for i in 0..node.children.len() 
                {
                    subterms.push(node.children[i].evaluate(lhs, tp));
                }

                tp.create(&node.head, &subterms)
            }
            Compressed(ct) => { ct.clone() }
            Variable(p) => { get_position(&lhs, &p).clone() }
        }
    }

    /// Creates a SCTT from a term. The var_map parameter should specify where which variable can be
    /// found in the lhs of the rewrite rule.
    pub(crate) fn from_term(t: &ATerm, tp: &TermPool, var_map: &HashMap<Symbol, ExplicitPosition>) -> SemiCompressedTermTree 
    {
        if t.arguments().len() == 0 
        {
            if var_map.contains_key(&t.get_head_symbol()) 
            {
                Variable(var_map.get(&t.get_head_symbol()).unwrap().clone())
            } 
            else 
            {
                Compressed(t.clone())
            }
        } 
        else 
        {
            let children = t.arguments().iter().map(|c| SemiCompressedTermTree::from_term(c, tp, var_map)).collect();
            let node = ExplicitNode{ head: t.get_head_symbol(), children };

            if node.children.iter().all(|c| c.is_compressed()) 
            {
                Compressed(t.clone())
            } 
            else 
            {
                Explicit(node)
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

    /// Get all references to all variables, contains duplicates
    fn get_all_var_references(&self) -> Vec<ExplicitPosition> 
    {
        let mut result = vec![];
        match self {
            Explicit(en) => {
                for n in &en.children 
                {
                    result.extend_from_slice(&n.get_all_var_references());
                }
            }
            Compressed(_) => {}
            Variable(ep) => { result.push(ep.clone()); }
        }

        result
    }
}

/// Create a mapping of variables to their position in the given term
pub(crate) fn create_var_map(t: &ATerm) -> HashMap<Symbol,ExplicitPosition> 
{
    // Queue of pairs of subterm and position in term t that need to be inspected
    let mut queue: VecDeque<(ATerm, ExplicitPosition)> = VecDeque::new();
    queue.push_back((t.clone(), ExplicitPosition::empty_pos()));

    let mut map = HashMap::new();
    while !queue.is_empty() 
    {
        // get a subterm to inspect
        let (term, pos) = queue.pop_front().unwrap();
        let head = term.get_head_symbol();

        if !term.is_variable() 
        {
            map.insert(head, pos.clone());
        }

        let mut i = 1;

        //Put subterms in the queue
        for sub in &term.arguments() 
        {
            let mut sub_pos = pos.clone();
            sub_pos.indices.push(i);
            queue.push_back((sub.clone(), sub_pos.clone()));

            i += 1;
        }
    }
    map
}

#[allow(unused_imports)]
mod tests {
    use super::*;    
    use mcrl2_rust::atermpp::{ATerm, TermPool};

    #[test]
    fn test_constant() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("a").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(&t, &tp, &map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn test_compressible() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(a,a)").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(&t, &tp, &map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn not_compressible() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(x,x)").unwrap();

        let map = create_var_map(&t);
        let sctt = SemiCompressedTermTree::from_term(&t, &tp, &map);

        let en = Explicit(ExplicitNode{
            head: tp.create_symbol("f", 2),
            children: vec![
                Variable(ExplicitPosition::new(&[1, 1])),
                Variable(ExplicitPosition::new(&[1, 2]))
            ] });

        assert_eq!(sctt,en);
    }

    #[test]
    fn partly_compressible() {
        let mut tp = TermPool::new();
        let tst = tp.from_string("f(f(a,a),x)").unwrap();
        let tst_compressible = tp.from_string("f(a,a)").unwrap();

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