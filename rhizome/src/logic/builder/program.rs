use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

use crate::{
    error::{error, Error},
    id::RelationId,
    logic::ast::{Clause, Declaration, Program, Rule},
    relation::{Edb, Idb},
};

use super::{
    declaration::DeclarationBuilder,
    fact::FactBuilder,
    rule::{RuleBodyBuilder, RuleHeadBuilder},
    rule_vars::RuleVars,
};

type RuleBuilderClosure<'a, T> = dyn Fn(
        RuleHeadBuilder<'a>,
        RuleBodyBuilder<'a>,
        &'_ T,
    ) -> (RuleHeadBuilder<'a>, RuleBodyBuilder<'a>)
    + 'a;

#[derive(Debug, Default)]
pub struct ProgramBuilder {
    relations: HashMap<String, Arc<Declaration>>,
    clauses: Vec<Clause>,
}

impl ProgramBuilder {
    pub fn build<F>(f: F) -> Result<Program>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let mut builder = Self::default();

        f(&mut builder)?;

        builder.finalize()
    }

    pub fn finalize(self) -> Result<Program> {
        let declarations = self.relations.into_values().collect();
        let program = Program::new(declarations, self.clauses);

        Ok(program)
    }

    pub fn input<F>(&mut self, id: &str, f: F) -> Result<()>
    where
        F: FnOnce(DeclarationBuilder<Edb>) -> DeclarationBuilder<Edb>,
    {
        if let Some(relation) = self.relations.get(&id.to_owned()) {
            return error(Error::ConflictingRelationDeclaration(relation.id()));
        }

        let rel_id = RelationId::new(id);
        let relation = DeclarationBuilder::build(rel_id, f)?;
        let relation = Declaration::Edb(relation);

        self.relations.insert(id.to_owned(), Arc::new(relation));

        Ok(())
    }

    pub fn output<F>(&mut self, id: &str, f: F) -> Result<()>
    where
        F: FnOnce(DeclarationBuilder<Idb>) -> DeclarationBuilder<Idb>,
    {
        if let Some(relation) = self.relations.get(&id.to_owned()) {
            return error(Error::ConflictingRelationDeclaration(relation.id()));
        }

        let rel_id = RelationId::new(id);
        let relation = DeclarationBuilder::build(rel_id, f)?;
        let relation = Declaration::Idb(relation);

        self.relations.insert(id.to_owned(), Arc::new(relation));

        Ok(())
    }

    pub fn fact<'b, F>(&'b mut self, id: &str, f: F) -> Result<()>
    where
        F: FnOnce(FactBuilder<'b>) -> FactBuilder<'b>,
    {
        let Some(declaration) = self.relations.get(id) else {
            return error(Error::UnrecognizedRelation(id.to_string()));
        };

        let fact = FactBuilder::build(declaration, f)?;
        let clause = Clause::Fact(fact);

        self.clauses.push(clause);

        Ok(())
    }

    pub fn rule<'a, T>(&'a mut self, id: &str, f: &RuleBuilderClosure<'a, T::Vars>) -> Result<()>
    where
        T: RuleVars,
    {
        let Some(declaration) = self.relations.get(id) else {
                return error(Error::UnrecognizedRelation(id.to_string()));
            };

        let mut bound_vars = HashMap::default();
        let head_builder = RuleHeadBuilder::new(declaration);
        let body_builder = RuleBodyBuilder::new(&self.relations);

        let (h, b) = f(head_builder, body_builder, &T::into_vars());

        let body = b.finalize(&mut bound_vars)?;
        let head = h.finalize(&bound_vars)?;

        match &**declaration {
            Declaration::Edb(inner) => error(Error::ClauseHeadEDB(inner.id())),
            Declaration::Idb(inner) => {
                let rule = Rule::new(inner.id(), head, body);
                let clause = Clause::Rule(rule);

                self.clauses.push(clause);

                Ok(())
            }
        }
    }
}