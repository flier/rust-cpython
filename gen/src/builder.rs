use syntex_syntax::ast;
use syntex_syntax::print::pprust;

use quote;

pub struct Builder {
    pub ident: ast::Ident,
    pub fields: Vec<ast::StructField>,
    pub methods: Vec<ast::ImplItem>,
}

impl Builder {
    pub fn class_name(&self) -> quote::Ident {
        quote::Ident::new(self.ident.to_string())
    }

    pub fn build(&self) -> quote::Tokens {
        let impl_to_py_object = self.impl_to_py_object();
        let impl_from_py_object = self.impl_from_py_object();
        let impl_python_object = self.impl_python_object();
        let impl_python_object_with_checked_downcast =
            self.impl_python_object_with_checked_downcast();
        let impl_base_object = self.impl_base_object();
        let impl_class_create_instance = self.impl_class_create_instance();

        quote! {
            #impl_to_py_object
            #impl_from_py_object
            #impl_python_object
            #impl_python_object_with_checked_downcast
            #impl_base_object
            #impl_class_create_instance
        }
    }

    fn impl_to_py_object(&self) -> quote::Tokens {
        let class_name = self.class_name();

        quote! {
            impl ::cpython::ToPyObject for #class_name {
                type ObjectType = #class_name;

                #[inline]
                fn to_py_object(&self, py: ::cpython::Python) -> Self::ObjectType {
                    ::cpython::PyClone::clone_ref(self, py)
                }

                #[inline]
                fn into_py_object(self, _py: ::cpython::Python) -> Self::ObjectType {
                    self
                }

                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: ::cpython::Python, f: F) -> R
                    where F: FnOnce(*mut ::cpython::_detail::ffi::PyObject) -> R
                {
                    f(::cpython::PythonObject::as_object(self).as_ptr())
                }
            }
        }
    }

    fn impl_from_py_object(&self) -> quote::Tokens {
        let class_name = self.class_name();

        quote! {
            impl <'source> ::cpython::FromPyObject<'source> for #class_name {
                #[inline]
                fn extract(py: ::cpython::Python, obj: &'source ::cpython::PyObject) -> ::cpython::PyResult<#class_name> {
                    use ::cpython::PyClone;
                    Ok(obj.clone_ref(py).cast_into::<#class_name>(py)?)
                }
            }

            impl <'source> ::cpython::FromPyObject<'source> for &'source #class_name {
                #[inline]
                fn extract(py: ::cpython::Python, obj: &'source ::cpython::PyObject) -> ::cpython::PyResult<&'source #class_name> {
                    Ok(obj.cast_as::<#class_name>(py)?)
                }
            }
        }
    }

    fn impl_python_object(&self) -> quote::Tokens {
        let class_name = self.class_name();

        quote! {
            impl ::cpython::PythonObject for #class_name {
                #[inline]
                fn as_object(&self) -> &::cpython::PyObject {
                    &self._unsafe_inner
                }

                #[inline]
                fn into_object(self) -> ::cpython::PyObject {
                    self._unsafe_inner
                }

                #[inline]
                unsafe fn unchecked_downcast_from(obj: ::cpython::PyObject) -> Self {
                    #class_name { _unsafe_inner: obj }
                }

                #[inline]
                unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a ::cpython::PyObject) -> &'a Self {
                    ::std::mem::transmute(obj)
                }
            }
        }
    }

    fn impl_python_object_with_checked_downcast(&self) -> quote::Tokens {
        let class_name = self.class_name();

        quote! {
            impl ::cpython::PythonObjectWithCheckedDowncast for #class_name {
                #[inline]
                fn downcast_from<'p>(py: ::cpython::Python<'p>, obj: ::cpython::PyObject) ->
                    Result<#class_name, ::cpython::PythonObjectDowncastError<'p>>
                {
                    if py.get_type::<#class_name>().is_instance(py, &obj) {
                        Ok(#class_name { _unsafe_inner: obj })
                    } else {
                        Err(::cpython::PythonObjectDowncastError(py))
                    }
                }

                #[inline]
                fn downcast_borrow_from<'a, 'p>(py: ::cpython::Python<'p>, obj: &'a ::cpython::PyObject) ->
                    Result<&'a #class_name, ::cpython::PythonObjectDowncastError<'p>>
                {
                    if py.get_type::<#class_name>().is_instance(py, obj) {
                        unsafe { Ok(::std::mem::transmute(obj)) }
                    } else {
                        Err(::cpython::PythonObjectDowncastError(py))
                    }
                }
            }
        }
    }

    fn impl_base_object(&self) -> quote::Tokens {
        let class_name = self.class_name();
        let base_type = self.class_name();

        let init_types = self.fields
            .iter()
            .map(|field| {
                let property_type = quote::Ident::new(pprust::ty_to_string(&*field.ty));

                quote!{ #property_type }
            });

        let init_args = self.fields
            .iter()
            .map(|field| {
                let property_name = quote::Ident::new(field.ident.unwrap().to_string());

                quote!{ #property_name }
            });

        quote! {
            impl ::cpython::py_class::BaseObject for #class_name {
                type InitType = ( #(#init_types),* );

                #[inline]
                fn size() -> usize {
                    <::cpython::PyObject as ::cpython::py_class::BaseObject>::size()
                }

                unsafe fn alloc(
                    py: ::cpython::Python,
                    ty: &::cpython::PyType,
                    ( #(#init_args),* ): Self::InitType
                ) -> ::cpython::PyResult<::cpython::PyObject>
                {
                }

                unsafe fn dealloc(py: ::cpython::Python, obj: *mut ::cpython::_detail::ffi::PyObject) {
                }
            }
        }
    }

    fn impl_class_create_instance(&self) -> quote::Tokens {
        let class_name = self.class_name();

        let mut tokens = quote::Tokens::new();

        for field in self.fields.iter() {
            let field_ident = field.ident.unwrap();
            let field_name = quote::Ident::new(field_ident.to_string());
            let getter_name = quote::Ident::new(format!("get_{}", field_ident));
            let setter_name = quote::Ident::new(format!("set_{}", field_ident));
            let property_type = quote::Ident::new(pprust::ty_to_string(&*field.ty));

            let impl_property = quote!{
                fn #getter_name(&self) -> #property_type {
                    self.#field_name
                }

                fn #setter_name(&self, value: #property_type) {
                    self.#field_name = value;
                }
            };

            tokens.append(impl_property.as_str());
        }

        let decl_args = self.fields
            .iter()
            .map(|field| {
                let property_name = quote::Ident::new(field.ident.unwrap().to_string());
                let property_type = quote::Ident::new(pprust::ty_to_string(&*field.ty));

                quote!{ #property_name: #property_type }
            });

        let call_args = self.fields
            .iter()
            .map(|field| {
                let property_name = quote::Ident::new(field.ident.unwrap().to_string());

                quote!{ #property_name }
            });

        let py_class_type_object_static_init = quote! {
            ::cpython::_detail::ffi::PyTypeObject {
                tp_dealloc: Some(::cpython::py_class::slots::tp_dealloc_callback::<#class_name>),
                tp_flags: ::cpython::py_class::slots::TPFLAGS_DEFAULT,
                ..
                ::cpython::_detail::ffi::PyTypeObject_INIT
            }
        };

        let py_class_type_object_dynamic_init = quote! {
            unsafe {
                TYPE_OBJECT.tp_name = concat!(stringify!(#class_name), "\0").as_ptr() as *const _;
                TYPE_OBJECT.tp_basicsize = <#class_name as ::cpython::py_class::BaseObject>::size()
                                            as ::cpython::_detail::ffi::Py_ssize_t;
            }
        };

        let init_members = self.fields.iter().map(|field| {
            let property_name = quote::Ident::new(field.ident.unwrap().to_string());

            quote! {
                dict.set_item(py, stringify!(#property_name), unsafe {
                    ::cpython::py_class::members::TypeMember::<#class_name>::into_descriptor(#property_name, py, &mut TYPE_OBJECT)?
                })?;
            }
        });

        let py_class_init_members = quote! {
            let dict = ::cpython::PyDict::new(py);

            #(#init_members)*

            unsafe {
                assert!(TYPE_OBJECT.tp_dict.is_null());
                TYPE_OBJECT.tp_dict = ::cpython::PythonObject::into_object(dict).steal_ptr();
            }
        };

        let impl_python_object_with_type_object = quote! {
            // trait implementations that need direct access to TYPE_OBJECT
            impl ::cpython::PythonObjectWithTypeObject for #class_name {
                fn type_object(py: ::cpython::Python) -> ::cpython::PyType {
                    unsafe {
                        if ::cpython::py_class::is_ready(py, &TYPE_OBJECT) {
                            ::cpython::PyType::from_type_ptr(py, &mut TYPE_OBJECT)
                        } else {
                            // automatically initialize the class on-demand
                            <#class_name as ::cpython::py_class::PythonObjectFromPyClassMacro>::initialize(py)
                                .expect(concat!("An error occurred while initializing class ", stringify!(#class_name)))
                        }
                    }
                }
            }
        };

        let impl_python_object_from_py_class_macro = quote! {
            impl ::cpython::py_class::PythonObjectFromPyClassMacro for #class_name {
                fn initialize(py: ::cpython::Python) -> ::cpython::PyResult<::cpython::PyType> {
                    unsafe {
                        if ::cpython::py_class::is_ready(py, &TYPE_OBJECT) {
                            return Ok(::cpython::PyType::from_type_ptr(py, &mut TYPE_OBJECT));
                        }
                        assert!(!INIT_ACTIVE,
                            concat!("Reentrancy detected: already initializing class ",
                            stringify!(#class_name)));
                        INIT_ACTIVE = true;
                        let res = init(py);
                        INIT_ACTIVE = false;
                        res
                    }
                }
            }
        };

        let impl_init = quote! {
            fn init(py: ::cpython::Python) -> ::cpython::PyResult<::cpython::PyType> {
                #py_class_type_object_dynamic_init
                #py_class_init_members
                unsafe {
                    if ::cpython::_detail::ffi::PyType_Ready(&mut TYPE_OBJECT) == 0 {
                        Ok(::cpython::PyType::from_type_ptr(py, &mut TYPE_OBJECT))
                    } else {
                        Err(::cpython::PyErr::fetch(py))
                    }
                }
            }
        };

        quote! {
            impl #class_name {
                #tokens

                fn create_instance(py: ::cpython::Python, #(#decl_args),*) -> ::cpython::PyResult<#class_name> {
                    let obj = unsafe {
                        <#class_name as ::cpython::py_class::BaseObject>::alloc(
                            py, &py.get_type::<#class_name>(), ( #(#call_args),*) )?
                    };
                    return Ok(#class_name { _unsafe_inner: obj });

                    static mut TYPE_OBJECT : ::cpython::_detail::ffi::PyTypeObject = #py_class_type_object_static_init;
                    static mut INIT_ACTIVE: bool = false;

                    #impl_python_object_with_type_object
                    #impl_python_object_from_py_class_macro
                    #impl_init
                }
            }
        }
    }
}
