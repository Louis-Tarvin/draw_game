import React, { useState } from 'react';

export default function useInput({ defaultValue = '', placeholder = '', type = 'text', ref = null, properties = {} } = {}) {
  let [value, setValue] = useState(defaultValue);

  const field = <input
    value={value}
    type={type}
    placeholder={placeholder}
    onChange={e => setValue(e.target.value)}
    ref={ref}
    {...properties} />;

  return [value, field];
}
