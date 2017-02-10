use quote;

use syntex_syntax::ast;

use super::errors::*;

pub struct Builder {
    ident: ast::Ident,
}

impl Builder {
    pub fn class_name(&self) -> String {
        self.ident.name.to_string()
    }

    pub fn build(&self) -> quote::Tokens {
        let impl_to_py_object = self.impl_to_py_object();
        let impl_from_py_object = self.impl_from_py_object();
        let impl_python_object = self.impl_python_object();
        let impl_python_object_with_checked_downcast =
            self.impl_python_object_with_checked_downcast();

        quote! {
            #impl_to_py_object
            #impl_from_py_object
            #impl_python_object
            #impl_python_object_with_checked_downcast
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
                    Ok(try!(obj.clone_ref(py).cast_into::<#class_name>(py)))
                }
            }

            impl <'source> ::cpython::FromPyObject<'source> for &'source #class_name {
                #[inline]
                fn extract(py: ::cpython::Python, obj: &'source ::cpython::PyObject) -> ::cpython::PyResult<&'source #class_name> {
                    Ok(try!(obj.cast_as::<#class_name>(py)))
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
                fn downcast_from<'p>(py: ::cpython::Python<'p>, obj: ::cpython::PyObject) -> Result<#class_name, ::cpython::PythonObjectDowncastError<'p>> {
                    if py.get_type::<#class_name>().is_instance(py, &obj) {
                        Ok(#class_name { _unsafe_inner: obj })
                    } else {
                        Err(::cpython::PythonObjectDowncastError(py))
                    }
                }

                #[inline]
                fn downcast_borrow_from<'a, 'p>(py: ::cpython::Python<'p>, obj: &'a ::cpython::PyObject) -> Result<&'a #class_name, ::cpython::PythonObjectDowncastError<'p>> {
                    if py.get_type::<#class_name>().is_instance(py, obj) {
                        unsafe { Ok(::std::mem::transmute(obj)) }
                    } else {
                        Err(::cpython::PythonObjectDowncastError(py))
                    }
                }
            }
        }
    }
}
