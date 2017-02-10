use syn;
use quote;

use super::errors::*;

pub struct Builder {
    prefix: String,
    suffix: String,
    ast: syn::MacroInput,
}

impl Builder {
    pub fn parse(s: &str) -> Result<Self> {
        let ast = syn::parse_macro_input(s)?;

        Ok(Builder {
            prefix: String::from("Py"),
            suffix: String::new(),
            ast: ast,
        })
    }

    pub fn prefix(&self) -> quote::Ident {
        quote::Ident::from("cpython")
    }

    pub fn class_name(&self) -> quote::Ident {
        quote::Ident::from(format!("{}{}{}", &self.prefix, &self.ast.ident, &self.suffix))
    }

    pub fn build(&self) -> quote::Tokens {
        let prefix = self.prefix();
        let class_name = self.class_name();

        let impl_to_py_object = self.impl_to_py_object();
        let impl_from_py_object = self.impl_from_py_object();
        let impl_python_object = self.impl_python_object();
        let impl_python_object_with_checked_downcast =
            self.impl_python_object_with_checked_downcast();

        quote! {
            struct #class_name { _unsafe_inner: ::#prefix::PyObject }

            #impl_to_py_object
            #impl_from_py_object
            #impl_python_object
            #impl_python_object_with_checked_downcast
        }
    }

    fn impl_to_py_object(&self) -> quote::Tokens {
        let prefix = self.prefix();
        let class_name = self.class_name();

        quote! {
            impl ::#prefix::ToPyObject for #class_name {
                type ObjectType = #class_name;

                #[inline]
                fn to_py_object(&self, py: ::#prefix::Python) -> Self::ObjectType {
                    ::#prefix::PyClone::clone_ref(self, py)
                }

                #[inline]
                fn into_py_object(self, _py: ::#prefix::Python) -> Self::ObjectType {
                    self
                }

                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: ::#prefix::Python, f: F) -> R
                    where F: FnOnce(*mut ::#prefix::_detail::ffi::PyObject) -> R
                {
                    f(::#prefix::PythonObject::as_object(self).as_ptr())
                }
            }
        }
    }

    fn impl_from_py_object(&self) -> quote::Tokens {
        let prefix = self.prefix();
        let class_name = self.class_name();

        quote! {
            impl <'source> ::#prefix::FromPyObject<'source> for #class_name {
                #[inline]
                fn extract(py: ::#prefix::Python, obj: &'source ::#prefix::PyObject) -> ::#prefix::PyResult<#class_name> {
                    use ::#prefix::PyClone;
                    Ok(try!(obj.clone_ref(py).cast_into::<#class_name>(py)))
                }
            }

            impl <'source> ::#prefix::FromPyObject<'source> for &'source #class_name {
                #[inline]
                fn extract(py: ::#prefix::Python, obj: &'source ::#prefix::PyObject) -> ::#prefix::PyResult<&'source #class_name> {
                    Ok(try!(obj.cast_as::<#class_name>(py)))
                }
            }
        }
    }

    fn impl_python_object(&self) -> quote::Tokens {
        let prefix = self.prefix();
        let class_name = self.class_name();

        quote! {
            impl ::#prefix::PythonObject for #class_name {
                #[inline]
                fn as_object(&self) -> &::#prefix::PyObject {
                    &self._unsafe_inner
                }

                #[inline]
                fn into_object(self) -> ::#prefix::PyObject {
                    self._unsafe_inner
                }

                #[inline]
                unsafe fn unchecked_downcast_from(obj: ::#prefix::PyObject) -> Self {
                    #class_name { _unsafe_inner: obj }
                }

                #[inline]
                unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a ::#prefix::PyObject) -> &'a Self {
                    ::std::mem::transmute(obj)
                }
            }
        }
    }

    fn impl_python_object_with_checked_downcast(&self) -> quote::Tokens {
        let prefix = self.prefix();
        let class_name = self.class_name();

        quote! {
            impl ::#prefix::PythonObjectWithCheckedDowncast for #class_name {
                #[inline]
                fn downcast_from<'p>(py: ::#prefix::Python<'p>, obj: ::#prefix::PyObject) -> Result<#class_name, ::#prefix::PythonObjectDowncastError<'p>> {
                    if py.get_type::<#class_name>().is_instance(py, &obj) {
                        Ok(#class_name { _unsafe_inner: obj })
                    } else {
                        Err(::#prefix::PythonObjectDowncastError(py))
                    }
                }

                #[inline]
                fn downcast_borrow_from<'a, 'p>(py: ::#prefix::Python<'p>, obj: &'a ::#prefix::PyObject) -> Result<&'a #class_name, ::#prefix::PythonObjectDowncastError<'p>> {
                    if py.get_type::<#class_name>().is_instance(py, obj) {
                        unsafe { Ok(::std::mem::transmute(obj)) }
                    } else {
                        Err(::#prefix::PythonObjectDowncastError(py))
                    }
                }
            }
        }
    }
}
