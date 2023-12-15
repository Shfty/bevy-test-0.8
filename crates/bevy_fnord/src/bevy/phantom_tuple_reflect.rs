#[macro_export]
macro_rules ! phantom_tuple_reflect {
    ($ident:ident, $storage:ty) => {
        impl<T> Component for $ident<T>
        where
            T: 'static + Send + Sync,
        {
            type Storage = $storage;
        }

        impl<T> TupleStruct for $ident<T>
        where
            T: 'static + Send + Sync + Serialize,
        {
            fn field(&self, _: usize) -> Option<&dyn Reflect> {
                None
            }

            fn field_mut(&mut self, _: usize) -> Option<&mut dyn Reflect> {
                None
            }

            fn field_len(&self) -> usize {
                0
            }

            fn iter_fields(&self) -> TupleStructFieldIter {
                TupleStructFieldIter::new(self)
            }

            fn clone_dynamic(&self) -> DynamicTupleStruct {
                let mut tuple: DynamicTupleStruct = default();
                tuple.set_name(std::any::type_name::<Self>().into());
                tuple
            }
        }

        unsafe impl<T> Reflect for $ident<T>
        where
            T: 'static + Send + Sync + Serialize,
        {
            fn type_name(&self) -> &str {
                std::any::type_name::<Self>()
            }

            fn any(&self) -> &dyn std::any::Any {
                self
            }

            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn apply(&mut self, _: &dyn Reflect) {}

            fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
                Err(value)
            }

            fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
                ReflectRef::TupleStruct(self)
            }

            fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
                ReflectMut::TupleStruct(self)
            }

            fn clone_value(&self) -> Box<dyn Reflect> {
                Box::new(self.clone_dynamic())
            }

            fn reflect_hash(&self) -> Option<u64> {
                None
            }

            fn reflect_partial_eq(&self, _value: &dyn Reflect) -> Option<bool> {
                None
            }

            fn serializable(&self) -> Option<Serializable> {
                Some(Serializable::Borrowed(self))
            }
        }

        impl<T> GetTypeRegistration for $ident<T>
        where
            T: 'static + Send + Sync + Serialize,
        {
            fn get_type_registration() -> TypeRegistration {
                let mut reg = TypeRegistration::of::<Self>();
                reg.insert(<ReflectComponent as FromType<Self>>::from_type());
                reg
            }
        }
    };
}

