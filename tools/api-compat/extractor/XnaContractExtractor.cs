// SPDX-License-Identifier: MS-PL
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Web.Script.Serialization;

internal static class XnaContractExtractor
{
    private const BindingFlags Declared = BindingFlags.DeclaredOnly | BindingFlags.Public |
        BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static;
    private static string referenceDirectory = "";

    public static int Main(string[] args)
    {
        if (args.Length < 3) return 2;
        referenceDirectory = Path.GetFullPath(args[0]);
        AppDomain.CurrentDomain.AssemblyResolve += ResolveAssembly;
        var types = new Dictionary<string, Type>(StringComparer.Ordinal);
        foreach (string name in args.Skip(2))
        {
            Assembly assembly = Assembly.LoadFrom(Path.Combine(referenceDirectory, name));
            foreach (Type type in SafeTypes(assembly))
                if (Visible(type) && type.FullName != null &&
                    type.FullName.StartsWith("Microsoft.Xna.Framework", StringComparison.Ordinal))
                    types[type.FullName] = type;
        }
        var contract = types.Values.OrderBy(t => t.FullName, StringComparer.Ordinal).Select(ReadType).ToList();
        var root = new Dictionary<string, object> {
            ["schemaVersion"] = 1, ["profile"] = "XNA 4.0 Windows runtime", ["types"] = contract
        };
        var serializer = new JavaScriptSerializer { MaxJsonLength = Int32.MaxValue, RecursionLimit = 256 };
        File.WriteAllText(args[1], serializer.Serialize(root));
        Console.WriteLine("REFERENCE_TYPES=" + contract.Count);
        Console.WriteLine("REFERENCE_MEMBERS=" + contract.Sum(t => ((List<object>)t["members"]).Count));
        return 0;
    }

    private static Assembly ResolveAssembly(object sender, ResolveEventArgs args)
    {
        string path = Path.Combine(referenceDirectory, new AssemblyName(args.Name).Name + ".dll");
        return File.Exists(path) ? Assembly.LoadFrom(path) : null;
    }

    private static IEnumerable<Type> SafeTypes(Assembly assembly)
    {
        try { return assembly.GetTypes(); }
        catch (ReflectionTypeLoadException error) { return error.Types.Where(t => t != null); }
    }

    private static bool Visible(Type type)
    {
        if (type.IsPublic) return true;
        return (type.IsNestedPublic || type.IsNestedFamily || type.IsNestedFamORAssem) &&
            type.DeclaringType != null && Visible(type.DeclaringType);
    }

    private static Dictionary<string, object> ReadType(Type type)
    {
        var members = new List<object>();
        foreach (ConstructorInfo value in type.GetConstructors(Declared).Where(Visible))
            members.Add(Callable("constructor", ".ctor", value));
        foreach (MethodInfo value in type.GetMethods(Declared).Where(Visible))
            if (!value.IsSpecialName || value.Name.StartsWith("op_", StringComparison.Ordinal))
                members.Add(Callable("method", value.Name, value));
        foreach (PropertyInfo value in type.GetProperties(Declared))
        {
            MethodInfo getter = value.GetGetMethod(true), setter = value.GetSetMethod(true);
            if ((getter != null && Visible(getter)) || (setter != null && Visible(setter)))
                members.Add(new Dictionary<string, object> {
                    ["kind"] = "property", ["name"] = value.Name, ["type"] = TypeName(value.PropertyType),
                    ["static"] = (getter ?? setter).IsStatic,
                    ["get"] = getter != null && Visible(getter), ["set"] = setter != null && Visible(setter),
                    ["parameters"] = value.GetIndexParameters().Select(Parameter).ToList()
                });
        }
        foreach (EventInfo value in type.GetEvents(Declared))
        {
            MethodInfo adder = value.GetAddMethod(true), remover = value.GetRemoveMethod(true);
            if ((adder != null && Visible(adder)) || (remover != null && Visible(remover)))
                members.Add(new Dictionary<string, object> { ["kind"] = "event", ["name"] = value.Name, ["type"] = TypeName(value.EventHandlerType) });
        }
        foreach (FieldInfo value in type.GetFields(Declared).Where(Visible))
            members.Add(new Dictionary<string, object> {
                ["kind"] = "field", ["name"] = value.Name, ["type"] = TypeName(value.FieldType),
                ["static"] = value.IsStatic, ["constant"] = value.IsLiteral
            });
        return new Dictionary<string, object> {
            ["name"] = type.FullName, ["kind"] = Kind(type),
            ["flags"] = type.IsEnum && type.IsDefined(typeof(FlagsAttribute), false),
            ["baseType"] = TypeName(type.BaseType), ["interfaces"] = type.GetInterfaces().Select(TypeName).ToList(),
            ["members"] = members
        };
    }

    private static Dictionary<string, object> Callable(string kind, string name, MethodBase value)
    {
        var method = value as MethodInfo;
        return new Dictionary<string, object> {
            ["kind"] = kind, ["name"] = name, ["static"] = value.IsStatic,
            ["returnType"] = method == null ? null : TypeName(method.ReturnType),
            ["parameters"] = value.GetParameters().Select(Parameter).ToList()
        };
    }

    private static Dictionary<string, object> Parameter(ParameterInfo value)
    {
        return new Dictionary<string, object> {
            ["name"] = value.Name ?? "", ["type"] = TypeName(value.ParameterType),
            ["out"] = value.IsOut, ["optional"] = value.IsOptional
        };
    }

    private static string Kind(Type type)
    {
        if (type.IsEnum) return "enum";
        if (type.IsInterface) return "interface";
        if (type.BaseType == typeof(MulticastDelegate)) return "delegate";
        if (type.IsValueType) return "struct";
        return "class";
    }

    private static bool Visible(MethodBase value) { return value.IsPublic || value.IsFamily || value.IsFamilyOrAssembly; }
    private static bool Visible(FieldInfo value) { return value.IsPublic || value.IsFamily || value.IsFamilyOrAssembly; }

    private static string TypeName(Type type)
    {
        if (type == null) return null;
        if (type.IsByRef) return TypeName(type.GetElementType()) + "&";
        if (type.IsArray) return TypeName(type.GetElementType()) + "[]";
        if (type.IsGenericParameter) return "!" + type.GenericParameterPosition;
        if (type.IsGenericType)
            return type.GetGenericTypeDefinition().FullName + "[" + String.Join(",", type.GetGenericArguments().Select(TypeName)) + "]";
        return type.FullName ?? type.Name;
    }
}
