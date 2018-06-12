use core::backend::Database;
use core::backend::Error;
use core::FromSql;
use core::query::condition::And;
use core::query::condition::Condition;
use core::query::Query;
use core::query::QueryPart;
use core::table::Field;
use core::table::Table;
use core::query::condition::Or;

#[derive(Debug)]
pub struct Join<'a, DB: Database + 'a> {
    pub other: &'a Table,
    pub join_condition: &'a dyn Condition<DB>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct OrderBy<'a> {
    pub field: &'a Field,
    pub ordering: Ordering,
}

#[derive(Debug)]
pub struct Select<'a, DB: Database + 'a> {
    pub table: &'a Table,
    pub fields: Vec<&'a Field>,
    pub joins: Vec<Join<'a, DB>>,
    pub filter: Option<Box<dyn Condition<DB>>>,
    pub subset: (Option<usize>, Option<usize>),
    pub order: Vec<OrderBy<'a>>,
}

impl<'a, DB: Database + 'a> Select<'a, DB> {
    pub fn new(table: &'a Table, fields: Vec<&'a Field>) -> Select<'a, DB> {
        Select {
            table,
            fields,
            joins: Vec::new(),
            filter: None,
            subset: (None, None),
            order: Vec::new(),
        }
    }

    pub fn limit(mut self, limit: usize) -> Select<'a, DB> {
        self.subset = (self.subset.0, Some(limit));
        self
    }

    pub fn offset(mut self, offset: usize) -> Select<'a, DB> {
        self.subset = (Some(offset), self.subset.1);
        self
    }

    pub fn select(mut self, fields: Vec<&'a Field>) -> Select<'a, DB> {
        self.fields = fields;
        self
    }

    pub fn order_by(mut self, field: &'a Field, ordering: Ordering) -> Select<'a, DB> {
        self.order.push(OrderBy { field, ordering });
        self
    }

    pub fn filter<Cond>(mut self, cond: Cond) -> Select<'a, DB>
        where
            DB: 'static,
            Cond: Condition<DB> + 'static,
            And<DB>: Condition<DB>
    {
        self.filter = match self.filter {
            None => Some(Box::new(cond)),
            Some(old) => Some(Box::new(And {
                cond_1: old,
                cond_2: Box::new(cond),
            }))
        };

        self
    }

    pub fn or<Cond>(mut self, cond: Cond) -> Select<'a, DB>
        where
            DB: 'static,
            Cond: Condition<DB> + 'static,
            Or<DB>: Condition<DB>
    {
        self.filter = match self.filter {
            None => Some(Box::new(cond)),
            Some(old) => Some(Box::new(Or {
                cond_1: old,
                cond_2: Box::new(cond),
            }))
        };

        self
    }
}

#[derive(Debug)]
pub struct Row<DB: Database> {
    fields: Vec<DB::Types>,
}

impl<DB: Database> Row<DB> {
    pub fn new(values: Vec<DB::Types>) -> Row<DB> {
        Row {
            fields: values
        }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<T>(&self, idx: isize) -> Option<Result<T, Error<DB>>>
        where
            T: FromSql<DB>
    {
        let idx: usize = if idx < 0 {
            (self.fields.len() as isize + idx) as usize
        } else {
            idx as usize
        };

        self.fields.get(idx).map(T::from_sql)
    }
}

pub trait Queryable<DB: Database>: Sized {
    fn select_from(table: &Table) -> Select<DB> {
        table.select()
    }

    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>>;
}

impl<'a, DB: Database + 'a> Query<DB> for Select<'a, DB>
    where
        Select<'a, DB>: QueryPart<DB> {}