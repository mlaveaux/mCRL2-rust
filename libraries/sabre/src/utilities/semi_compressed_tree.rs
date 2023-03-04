// Author(s): Mark Bouwman

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
    /// Returns true iff this tree is compressed.
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
    pub(crate) fn from_term(t: ATerm, var_map: &HashMap<Symbol, ExplicitPosition>) -> SemiCompressedTermTree 
    {
        if t.arguments().is_empty()
        {
            if var_map.contains_key(&t.get_head_symbol()) 
            {
                Variable(var_map.get(&t.get_head_symbol()).unwrap().clone())
            } 
            else 
            {
                Compressed(t)
            }
        } 
        else 
        {
            let children = t.arguments().iter().map(|c| SemiCompressedTermTree::from_term(c.clone(), var_map)).collect();
            let node = ExplicitNode{ head: t.get_head_symbol(), children };

            if node.children.iter().all(|c| c.is_compressed()) 
            {
                Compressed(t)
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

        for r in references 
        {
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

        if term.is_variable() 
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

#[cfg(test)]
mod tests 
{
    use super::*;    
    use mcrl2_rust::atermpp::{TermPool};

    #[test]
    fn test_constant() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("a").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(t.clone(),  &map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn test_compressible() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(a,a)").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(t.clone(), &&map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn test_not_compressible() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(x,x)").unwrap();
        let f = tp.create_symbol("f", 2);

        let mut map = HashMap::new();
        map.insert(f.clone(), ExplicitPosition::new(&[2]));

        let sctt = SemiCompressedTermTree::from_term(t, &map);

        let en = Explicit(ExplicitNode{
            head: f,
            children: vec![
                Variable(ExplicitPosition::new(&[2])), // Note that both point to the second occurence of x.
                Variable(ExplicitPosition::new(&[2]))
            ] });

        assert_eq!(sctt,en);
    }

    #[test]
    fn test_partly_compressible() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(f(a,a), x)").unwrap();
        let compressible = tp.from_string("f(a,a)").unwrap();

        // Make a variable map with only x@2.        
        let mut map = HashMap::new();
        map.insert(tp.create_symbol("x", 0), ExplicitPosition::new(&[2]));

        let sctt = SemiCompressedTermTree::from_term(t, &map);
        let en = Explicit(ExplicitNode{
            head: tp.create_symbol("f", 2),
            children: vec![
                Compressed(compressible),
                Variable(ExplicitPosition::new(&[2]))
            ] });
        assert_eq!(sctt,en);
    }

    #[test]
    fn test_evaluation() 
    {
        let mut tp = TermPool::new();
        let t_rhs = tp.from_string("f(f(a,a),x)").unwrap();
        let t_lhs = tp.from_string("g(b)").unwrap();
        
        // Make a variable map with only x@2.       
        let mut map = HashMap::new();
        map.insert(tp.create_symbol("x", 0), ExplicitPosition::new(&[2]));

        let sctt = SemiCompressedTermTree::from_term(t_rhs, &map);

        let t_expected = tp.from_string("f(f(a,a),b)").unwrap();
        assert_eq!(sctt.evaluate(&t_lhs, &mut tp), t_expected);
    }

    #[test]
    fn test_create_varmap() 
    {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(x,x)").unwrap();
        let x = tp.create_symbol("x", 0);

        let map = create_var_map(&t);
        println!("{:?}", map);
        assert!(map.contains_key(&x));
    }
}