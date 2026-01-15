pub struct FromInherited<Inh, Syn> {
    mapper: Box<dyn FnOnce(Inh) -> Syn>,
}

impl<Inh: 'static, Syn: 'static> FromInherited<Inh, Syn> {
    pub fn new<F>(mapper: F) -> Self
    where
        F: FnOnce(Inh) -> Syn + 'static,
    {
        Self {
            mapper: Box::new(mapper),
        }
    }

    pub fn inspect_inherited<F>(self, clos: F) -> Self
    where
        F: FnOnce(&Inh) + 'static,
    {
        FromInherited::new(|t| {
            clos(&t);
            self.resolve(t)
        })
    }

    pub fn inspect_synthesized<F>(self, clos: F) -> Self
    where
        F: FnOnce(&Syn) + 'static,
    {
        FromInherited::new(|t| {
            let out = self.resolve(t);
            clos(&out);
            out
        })
    }

    pub fn inherit<ParentInh, F>(self, mapper: F) -> FromInherited<ParentInh, Syn>
    where
        ParentInh: 'static,
        F: FnOnce(ParentInh) -> Inh + 'static,
    {
        FromInherited::new(|t| self.resolve(mapper(t)))
    }

    pub fn map<ParentSyn, G>(self, mapper: G) -> FromInherited<Inh, ParentSyn>
    where
        ParentSyn: 'static,
        G: FnOnce(Syn) -> ParentSyn + 'static,
    {
        FromInherited::new(|t| mapper(self.resolve(t)))
    }

    pub fn synthesize<G, ParentSyn>(self, mapper: G) -> FromInherited<Inh, ParentSyn>
    where
        Inh: Clone,
        ParentSyn: 'static,
        G: FnOnce(Inh, Syn) -> ParentSyn + 'static,
    {
        FromInherited::new(|input: Inh| {
            let input_clone = input.clone();
            mapper(input_clone, self.resolve(input))
        })
    }

    pub fn inherit_ref<ParentInh, F>(self, mapper: F) -> FromInherited<ParentInh, (ParentInh, Syn)>
    where
        ParentInh: 'static,
        F: FnOnce(&ParentInh) -> Inh + 'static,
    {
        FromInherited::new(|inherited| {
            let new_in = mapper(&inherited);
            (inherited, self.resolve(new_in))
        })
    }

    pub fn chain<Final>(self, next: FromInherited<Syn, Final>) -> FromInherited<Inh, Final>
    where
        Final: 'static,
    {
        FromInherited::new(|input| next.resolve(self.resolve(input)))
    }

    pub fn zip<OtherInh, OtherSyn>(
        self,
        other: FromInherited<OtherInh, OtherSyn>,
    ) -> FromInherited<(Inh, OtherInh), (Syn, OtherSyn)>
    where
        OtherInh: 'static,
        OtherSyn: 'static,
    {
        FromInherited::new(|(t, t1)| (self.resolve(t), other.resolve(t1)))
    }

    pub fn split<OtherSyn>(
        self,
        other: FromInherited<Inh, OtherSyn>,
    ) -> FromInherited<Inh, (Syn, OtherSyn)>
    where
        Inh: Clone,
        OtherSyn: 'static,
        OtherSyn: 'static,
    {
        FromInherited::new(|t: Inh| (self.resolve(t.clone()), other.resolve(t)))
    }

    pub fn resolve(self, value: Inh) -> Syn {
        (self.mapper)(value)
    }
}
